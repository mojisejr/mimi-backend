//! Test TarotQueue Missing Methods
//!
//! Tests for poll_next_job() and update_job_status() methods in TarotQueue.
//! These tests are written before implementation (Red-Green-Refactor TDD).

use mimivibe_backend::queue::TarotQueue;
use uuid::Uuid;
use serde_json::json;

mod setup;  // AUTO-LOAD .env via tests/setup.rs

#[tokio::test]
async fn test_poll_next_job_empty_queue() {
    setup::setup();

    let queue = TarotQueue::from_env().await.unwrap();

    // Ensure no jobs in queue
    sqlx::query!("DELETE FROM jobs WHERE status = 'queued'")
        .execute(&queue.db_pool)
        .await
        .unwrap();

    // Poll should return None
    let job = queue.poll_next_job().await.unwrap();
    assert!(job.is_none(), "Expected None when no jobs in queue");
}

#[tokio::test]
async fn test_poll_next_job_with_jobs() {
    setup::setup();

    let queue = TarotQueue::from_env().await.unwrap();

    // Clean up first
    sqlx::query!("DELETE FROM jobs WHERE status = 'queued'")
        .execute(&queue.db_pool)
        .await
        .unwrap();

    // Create test job with payload
    let job_id = Uuid::new_v4();
    let payload = json!({
        "question": "ความรักของฉันจะเป็นอย่างไร",
        "card_count": 3,
        "user_id": "test-user-123"
    });
    sqlx::query!(
        r#"
        INSERT INTO jobs (id, job_type, payload, status, created_at, updated_at)
        VALUES ($1, 'tarot_reading', $2, 'queued', NOW(), NOW())
        "#,
        job_id,
        payload
    )
    .execute(&queue.db_pool)
    .await
    .unwrap();

    // Poll job
    let job = queue.poll_next_job().await.unwrap();
    assert!(job.is_some(), "Expected a job when queue has jobs");
    let job = job.unwrap();
    assert_eq!(job.id, job_id);
    assert_eq!(job.question, "ความรักของฉันจะเป็นอย่างไร");
    assert_eq!(job.cards, 3);

    // Verify job is marked as processing
    let status = sqlx::query!(
        r#"SELECT status::text as status FROM jobs WHERE id = $1"#,
        job_id
    )
    .fetch_one(&queue.db_pool)
    .await
    .unwrap();
    assert_eq!(status.status.unwrap(), "processing");
}

#[tokio::test]
async fn test_update_job_status() {
    setup::setup();

    let queue = TarotQueue::from_env().await.unwrap();

    // Create test job
    let job_id = Uuid::new_v4();
    let payload = json!({
        "question": "อนาคตของฉันจะเป็นอย่างไร",
        "card_count": 5,
        "user_id": "test-user-456"
    });
    sqlx::query!(
        r#"
        INSERT INTO jobs (id, job_type, payload, status, created_at, updated_at)
        VALUES ($1, 'tarot_reading', $2, 'processing', NOW(), NOW())
        "#,
        job_id,
        payload
    )
    .execute(&queue.db_pool)
    .await
    .unwrap();

    // Update to completed without result
    queue.update_job_status(job_id, "completed", None).await.unwrap();

    // Verify final state
    let final_job = sqlx::query!(
        r#"SELECT status::text as status, completed_at FROM jobs WHERE id = $1"#,
        job_id
    )
    .fetch_one(&queue.db_pool)
    .await
    .unwrap();
    assert_eq!(final_job.status.unwrap(), "completed");
    assert!(final_job.completed_at.is_some());
}

#[tokio::test]
async fn test_update_job_status_with_result() {
    setup::setup();

    let queue = TarotQueue::from_env().await.unwrap();

    // Create test job
    let job_id = Uuid::new_v4();
    let payload = json!({
        "question": "การงานของฉันจะดีขึ้นหรือไม่",
        "card_count": 3,
        "user_id": "test-user-789"
    });
    sqlx::query!(
        r#"
        INSERT INTO jobs (id, job_type, payload, status, created_at, updated_at)
        VALUES ($1, 'tarot_reading', $2, 'processing', NOW(), NOW())
        "#,
        job_id,
        payload
    )
    .execute(&queue.db_pool)
    .await
    .unwrap();

    // Update to completed with result
    let result = json!({
        "header": "ทำนายไพ่ทาโรต์",
        "reading": "การงานของคุณจะดีขึ้นในเร็วๆ นี้",
        "cards": ["The Fool", "The Magician", "The Star"]
    });
    queue.update_job_status(job_id, "completed", Some(result.clone())).await.unwrap();

    // Verify final state
    let final_job = sqlx::query!(
        r#"SELECT status::text as status, result, completed_at FROM jobs WHERE id = $1"#,
        job_id
    )
    .fetch_one(&queue.db_pool)
    .await
    .unwrap();
    assert_eq!(final_job.status.unwrap(), "completed");
    assert!(final_job.result.is_some());
    assert_eq!(final_job.result.unwrap(), result);
    assert!(final_job.completed_at.is_some());
}

#[tokio::test]
async fn test_update_job_status_failed() {
    setup::setup();

    let queue = TarotQueue::from_env().await.unwrap();

    // Create test job
    let job_id = Uuid::new_v4();
    let payload = json!({
        "question": "สุขภาพของฉันเป็นอย่างไร",
        "card_count": 3,
        "user_id": "test-user-health"
    });
    sqlx::query!(
        r#"
        INSERT INTO jobs (id, job_type, payload, status, created_at, updated_at)
        VALUES ($1, 'tarot_reading', $2, 'processing', NOW(), NOW())
        "#,
        job_id,
        payload
    )
    .execute(&queue.db_pool)
    .await
    .unwrap();

    // Update to failed
    queue.update_job_status(job_id, "failed", None).await.unwrap();

    // Verify final state
    let final_job = sqlx::query!(
        r#"SELECT status::text as status FROM jobs WHERE id = $1"#,
        job_id
    )
    .fetch_one(&queue.db_pool)
    .await
    .unwrap();
    assert_eq!(final_job.status.unwrap(), "failed");
}

#[tokio::test]
async fn test_update_job_status_invalid_status() {
    setup::setup();

    let queue = TarotQueue::from_env().await.unwrap();

    // Create test job
    let job_id = Uuid::new_v4();
    let payload = json!({
        "question": "ครอบครัวของฉัน",
        "card_count": 3,
        "user_id": "test-user-family"
    });
    sqlx::query!(
        r#"
        INSERT INTO jobs (id, job_type, payload, status, created_at, updated_at)
        VALUES ($1, 'tarot_reading', $2, 'processing', NOW(), NOW())
        "#,
        job_id,
        payload
    )
    .execute(&queue.db_pool)
    .await
    .unwrap();

    // Try to update with invalid status
    let result = queue.update_job_status(job_id, "invalid_status", None).await;
    assert!(result.is_err(), "Expected error for invalid status");
}

#[tokio::test]
async fn test_full_job_lifecycle() {
    setup::setup();

    let queue = TarotQueue::from_env().await.unwrap();

    // Clean up first
    sqlx::query!("DELETE FROM jobs WHERE status = 'queued'")
        .execute(&queue.db_pool)
        .await
        .unwrap();

    // Create test job
    let job_id = Uuid::new_v4();
    let payload = json!({
        "question": "ชีวิตของฉันจะเปลี่ยนแปลงอย่างไร",
        "card_count": 5,
        "user_id": "test-user-life"
    });
    sqlx::query!(
        r#"
        INSERT INTO jobs (id, job_type, payload, status, created_at, updated_at)
        VALUES ($1, 'tarot_reading', $2, 'queued', NOW(), NOW())
        "#,
        job_id,
        payload
    )
    .execute(&queue.db_pool)
    .await
    .unwrap();

    // Poll job
    let job = queue.poll_next_job().await.unwrap();
    assert!(job.is_some(), "Expected a job when queue has jobs");
    let job = job.unwrap();
    assert_eq!(job.id, job_id);
    assert_eq!(job.question, "ชีวิตของฉันจะเปลี่ยนแปลงอย่างไร");
    assert_eq!(job.cards, 5);

    // Verify job is marked as processing
    let status = sqlx::query!(
        r#"SELECT status::text as status FROM jobs WHERE id = $1"#,
        job_id
    )
    .fetch_one(&queue.db_pool)
    .await
    .unwrap();
    assert_eq!(status.status.unwrap(), "processing");

    // Update to completed with result
    let result = json!({
        "header": "ทำนายไพ่ทาโรต์ 5 ใบ",
        "reading": "ชีวิตของคุณจะมีการเปลี่ยนแปลงครั้งใหญ่ในเดือนหน้า",
        "cards": ["The Fool", "The Tower", "The Star", "The Sun", "The World"]
    });
    queue.update_job_status(job_id, "completed", Some(result)).await.unwrap();

    // Verify final state
    let final_job = sqlx::query!(
        r#"SELECT status::text as status, result, completed_at FROM jobs WHERE id = $1"#,
        job_id
    )
    .fetch_one(&queue.db_pool)
    .await
    .unwrap();
    assert_eq!(final_job.status.unwrap(), "completed");
    assert!(final_job.result.is_some());
    assert!(final_job.completed_at.is_some());
}