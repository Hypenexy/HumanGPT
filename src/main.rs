use axum::{
    extract::Extension,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json,
    Router,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tower_http::services::ServeDir;
use dotenv::dotenv;

#[derive(sqlx::FromRow, Serialize)]
pub struct Prompt {
    pub id: i32,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: String, // "user" (the LLM's human persona) or "model" (the player)
    pub parts: Vec<Part>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Part {
    pub text: String,
}

// The structure we want the frontend to send to our Rust backend
#[derive(Deserialize, Debug)]
pub struct ChatRequest {
    pub history: Vec<Message>,
}

async fn post_text(Json(payload): Json<ChatRequest>) -> impl IntoResponse {
    match get_ai_response(payload.history).await {
        Ok(game_response) => Json(game_response).into_response(),
        Err(err) => {
            eprintln!("Gemini request failed: {err}");
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": err.to_string() })),
            )
                .into_response()
        }
    }
}

// The structured response we force the LLM to return
#[derive(Deserialize, Serialize, Debug)]
pub struct GameResponse {
    pub rating: u8,            // Score out of 10
    pub commentary: String,    // The "human" reacting to the player's AI response
    pub next_prompt: String,   // The next question the human asks the AI
}

async fn connect_db() -> sqlx::PgPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool.");
    
    println!("Connected to the database!");
    pool
}

async fn get_prompt(Extension(pool): Extension<sqlx::PgPool>) -> impl IntoResponse {
    let prompt = sqlx::query_as::<_, Prompt>(
        "SELECT id, value FROM prompts ORDER BY random() LIMIT 1",
    )
    .fetch_one(&pool)
    .await;

    match prompt {
        Ok(value) => Json(value).into_response(),
        Err(err) => {
            eprintln!("Failed to fetch prompt: {err}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn root() -> impl IntoResponse {
    match tokio::fs::read_to_string("dist/index.html").await {
        Ok(html) => Html(html).into_response(),
        Err(err) => {
            eprintln!("Failed to read dist/index.html: {err}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn get_ai_response(history: Vec<Message>) -> Result<GameResponse, Box<dyn std::error::Error>> {
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let client = Client::new();

    // System instructions telling the AI who it is and how to output data
    let system_instruction = "You are a user playing a game called HumanGPT. You are the HUMAN client, and the person replying to you is an AI chatbot. Evaluate their last response. Respond strictly in JSON matching this schema: { 'rating': number (1-10), 'commentary': 'your human reaction to their AI response', 'next_prompt': 'your next challenge/question for the AI' }";

    // Build the payload
    let payload = json!({
        "contents": history,
        "systemInstruction": {
            "parts": [{ "text": system_instruction }]
        },
        "generationConfig": {
            "responseMimeType": "application/json" // Enforces JSON output
        }
    });

    let response = client.post(&url)
        .header("Content-Type", "application/json")
        .body(payload.to_string())
        .send()
        .await?;

    let response_text = response.text().await?;
    let response_body: serde_json::Value = serde_json::from_str(&response_text)?;
    
    // Extract the text string containing the JSON from Gemini's nested response structure
    let raw_json_text = response_body["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or("Failed to get text from API response")?;

    // Parse that raw JSON string into our native GameResponse struct
    let game_data: GameResponse = serde_json::from_str(raw_json_text)?;

    Ok(game_data)
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let pool = connect_db().await;

    // build our application with a single route
    
    let app = Router::new()
        .route("/", get(root))
        .route("/sendresponse", post(post_text))
        .route("/getprompt", get(get_prompt))
        .fallback_service(ServeDir::new("dist"))
        .layer(Extension(pool));

    // run our app with hyper, listening globally on port 3000
    println!("Server listening, access it on http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}