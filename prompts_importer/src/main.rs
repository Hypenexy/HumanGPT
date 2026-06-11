use postgres::{Client, NoTls};
use std::fs::File;
use std::io::{self, BufRead};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let conn_str = std::env::var("DATABASE_URL").unwrap();
    let mut client = Client::connect(&conn_str, NoTls)?;

    client.execute(
        "CREATE TABLE IF NOT EXISTS prompts (
            id INT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        &[],
    )?;

    let file = File::open("../AiQueries.txt")?;
    let reader = io::BufReader::new(file);
    let mut tx = client.transaction()?;
    let stmt = tx.prepare("INSERT INTO prompts (id, value) VALUES ($1, $2)")?;

    println!("Migrating data...");

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        let id = (index + 1) as i32; 
        tx.execute(&stmt, &[&id, &line])?;
    }
    tx.commit()?;

    println!("Migration successfully completed!");
    Ok(())
}