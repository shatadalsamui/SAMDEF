use dotenv::dotenv;
use sqlx::PgPool;
use std::env;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Get database URL from environment
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env or environment");

    // Create a connection pool
    let pool = PgPool::connect(&database_url).await?;

    // Read the schema SQL file
    let schema_path = "src/schema/init.sql";
    let schema_sql = fs::read_to_string(schema_path).await?;

    // Split and execute each SQL statement
    for stmt in schema_sql.split(';') {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            println!("Attempting to execute:\n{}", trimmed); // Print the statement before execution
            match sqlx::query(trimmed).execute(&pool).await {
                Ok(_) => println!("Success."),
                Err(e) => {
                    eprintln!("Error executing statement:\n{}\nError: {}", trimmed, e);
                }
            }
        }
    }

    println!("Schema initialization complete!");

    Ok(())
}
