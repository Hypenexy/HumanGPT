use axum::{
    extract::Extension,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Json,
    Router,
};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tower_http::services::ServeDir;
use dotenv::dotenv;

#[derive(sqlx::FromRow, Serialize)]
pub struct Prompt {
    pub id: i32,
    pub prompt: String,
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
        "SELECT id, prompt FROM prompts ORDER BY random() LIMIT 1",
    )
    .fetch_one(&pool)
    .await;

    match prompt {
        Ok(prompt) => Json(prompt).into_response(),
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

#[tokio::main]
async fn main() {
    dotenv().ok();
    let pool = connect_db().await;

    // build our application with a single route
    
    let app = Router::new()
        .route("/", get(root))
        .fallback_service(ServeDir::new("dist"))
        .route("/getprompt", get(get_prompt))
        .layer(Extension(pool));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    
    println!("Server listening, access it on http://localhost:3000");
}