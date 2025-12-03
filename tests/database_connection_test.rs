//! Database Connection Testing
//!
//! Comprehensive testing for Supabase database connection issues
//! This will help identify where the connection problems occur

use sqlx::{PgPool, Row};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    println!("🔍 Database Connection Testing");
    println!("================================");

    // Test 1: Check environment variables
    println!("\n📋 Step 1: Environment Variables Check");
    match env::var("DATABASE_URL") {
        Ok(url) => {
            println!("✅ DATABASE_URL found: {}", url);

            // Analyze URL type
            if url.contains("db.") && url.contains(".supabase.co") {
                println!("🟡 Using DIRECT connection (db.xxx.supabase.co)");
                println!("   Consider using pooler for better performance");
            } else if url.contains("pooler.supabase.com") {
                println!("✅ Using POOLER connection (recommended)");
            } else {
                println!("❓ Unknown URL format");
            }
        }
        Err(_) => {
            println!("❌ DATABASE_URL not found");
            return Err("DATABASE_URL environment variable not set".into());
        }
    }

    // Test 2: Basic connection test
    println!("\n📋 Step 2: Basic Connection Test");
    match test_basic_connection().await {
        Ok(_) => println!("✅ Basic connection successful"),
        Err(e) => {
            println!("❌ Basic connection failed: {}", e);
            return Err(e);
        }
    }

    // Test 3: Connection pool test
    println!("\n📋 Step 3: Connection Pool Test");
    match test_connection_pool().await {
        Ok(_) => println!("✅ Connection pool successful"),
        Err(e) => {
            println!("❌ Connection pool failed: {}", e);
            return Err(e);
        }
    }

    // Test 4: Database query test
    println!("\n📋 Step 4: Database Query Test");
    match test_database_query().await {
        Ok(_) => println!("✅ Database query successful"),
        Err(e) => {
            println!("❌ Database query failed: {}", e);
            return Err(e);
        }
    }

    // Test 5: ReadingJob table test
    println!("\n📋 Step 5: ReadingJob Table Test");
    match test_reading_job_table().await {
        Ok(_) => println!("✅ ReadingJob table test successful"),
        Err(e) => {
            println!("❌ ReadingJob table test failed: {}", e);
            // Don't return error here - this might be due to table not existing
        }
    }

    println!("\n🎉 Database Connection Testing Complete!");
    println!("📝 Summary:");
    println!("   - Environment variables: ✅");
    println!("   - Basic connection: ✅");
    println!("   - Connection pool: ✅");
    println!("   - Database query: ✅");
    println!(
        "   - ReadingJob table: {}",
        if test_reading_job_table().await.is_ok() {
            "✅"
        } else {
            "⚠️"
        }
    );

    Ok(())
}

/// Test basic database connection
async fn test_basic_connection() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")?;

    println!(
        "   Testing connection to: {}",
        database_url.split('@').nth(1).unwrap_or("unknown")
    );

    // Try to connect with minimal configuration
    let pool = PgPool::connect(&database_url).await?;

    // Test simple connection
    let result = sqlx::query("SELECT 1 as test").fetch_one(&pool).await?;

    let test_value: i32 = result.get("test");
    if test_value == 1 {
        println!("   ✅ Query executed successfully");
    } else {
        return Err("Unexpected query result".into());
    }

    pool.close().await;
    Ok(())
}

/// Test connection pool configuration
async fn test_connection_pool() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")?;

    // Create connection pool with specific settings
    let pool = PgPool::connect(&database_url).await?;

    println!("   Pool connection established");

    // Test multiple connections
    let mut handles = vec![];
    for i in 0..5 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            let result = sqlx::query("SELECT $1 as test_id")
                .bind(i)
                .fetch_one(&pool_clone)
                .await;

            match result {
                Ok(row) => {
                    let test_id: i32 = row.get("test_id");
                    println!("   ✅ Pool connection {} successful", test_id);
                    Ok(())
                }
                Err(e) => {
                    println!("   ❌ Pool connection {} failed: {}", i, e);
                    Err(e)
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all connections to complete
    for handle in handles {
        handle.await??;
    }

    pool.close().await;
    Ok(())
}

/// Test database operations
async fn test_database_query() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;

    // Test current database and user separately
    let db_result = sqlx::query("SELECT current_database()")
        .fetch_one(&pool)
        .await?;
    let db_name: String = db_result.get(0);

    let user_result = sqlx::query("SELECT current_user()")
        .fetch_one(&pool)
        .await?;
    let user_name: String = user_result.get(0);

    println!("   ✅ Connected to database: {}", db_name);
    println!("   ✅ Connected as user: {}", user_name);

    // Test version
    let version_result = sqlx::query("SELECT version()").fetch_one(&pool).await?;

    let version: String = version_result.get(0);
    println!(
        "   ✅ PostgreSQL version: {}",
        version.split(',').next().unwrap_or("unknown")
    );

    pool.close().await;
    Ok(())
}

/// Test ReadingJob table specifically
async fn test_reading_job_table() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;

    // Check if table exists - note it's called "jobs" not "reading_jobs"
    let table_check = sqlx::query(
        "
        SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = 'jobs'
        ) as table_exists
    ",
    )
    .fetch_one(&pool)
    .await?;

    let table_exists: bool = table_check.get("table_exists");

    if !table_exists {
        println!("   ⚠️ Jobs table does not exist");
        println!("   💡 Run migrations first: sqlx migrate run");
        return Err("Jobs table not found".into());
    }

    println!("   ✅ Jobs table exists");

    // Test table structure
    let structure_check = sqlx::query(
        "
        SELECT column_name, data_type, is_nullable
        FROM information_schema.columns
        WHERE table_name = 'jobs'
        ORDER BY ordinal_position
    ",
    )
    .fetch_all(&pool)
    .await?;

    println!("   ✅ Jobs table structure:");
    for row in structure_check {
        let column: String = row.get("column_name");
        let data_type: String = row.get("data_type");
        let nullable: String = row.get("is_nullable");
        println!("      - {} ({} nullable: {})", column, data_type, nullable);
    }

    // Test inserting a dummy record
    let job_id = uuid::Uuid::new_v4();
    let insert_test = sqlx::query(
        "
        INSERT INTO jobs (id, job_type, payload, status, created_at)
        VALUES ($1, 'test', '{}', 'queued', NOW())
    ",
    )
    .bind(job_id)
    .execute(&pool)
    .await;

    match insert_test {
        Ok(_) => {
            println!("   ✅ Insert test successful");

            // Clean up test record
            let _ = sqlx::query("DELETE FROM jobs WHERE id = $1")
                .bind(job_id)
                .execute(&pool)
                .await;
        }
        Err(e) => {
            println!("   ❌ Insert test failed: {}", e);
            return Err(e.into());
        }
    }

    pool.close().await;
    Ok(())
}
