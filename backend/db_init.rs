// db_init.rs - Eagerly initializes the PostgreSQL schema on Neon Postgres
// Reads the wme_schema.sql file and executes it.

use std::env;
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    
    let db_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set in .env");
        
    println!("Connecting to PostgreSQL database...");
    let pool = sqlx::PgPool::connect(&db_url).await?;
    println!("Connected successfully.");

    // Locate the wme_schema.sql file. Check parent directory first, then local.
    let schema_path = if Path::new("../wme_schema.sql").exists() {
        "../wme_schema.sql".to_string()
    } else if Path::new("wme_schema.sql").exists() {
        "wme_schema.sql".to_string()
    } else {
        panic!("wme_schema.sql file not found in current or parent directory!");
    };

    println!("Reading schema from {}...", schema_path);
    let schema_sql = fs::read_to_string(&schema_path)?;

    println!("Executing schema migration...");
    sqlx::query(&schema_sql).execute(&pool).await?;

    println!("Schema migrated successfully! All tables created.");
    Ok(())
}
