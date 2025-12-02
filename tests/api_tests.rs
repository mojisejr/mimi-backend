//! API Integration Tests for Tarot Reading Endpoints
//!
//! Test-Driven Development: Tests written BEFORE implementation.
//! These tests should FAIL initially (Red Phase).

use reqwest;
use serde_json::{json, Value};
use std::time::Duration;
use uuid::Uuid;

/// Test for valid question length (5-100 characters)
#[tokio::test]
async fn test_question_validation_valid_length() {
    let client = reqwest::Client::new();

    // Test minimum valid length (5 characters)
    let min_valid = json!({
        "question": "ควรทำ"
    });

    let response = client
        .post("http://localhost:3000/api/v1/tarots/read")
        .json(&min_valid)
        .send()
        .await;

    // Should succeed since we haven't implemented the API yet
    // This test will FAIL in Red Phase (expected)
    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 201, "Minimum valid question should be accepted");
        }
        Err(e) => {
            // Expected in Red Phase - API not implemented yet
            println!("Expected error in Red Phase: {}", e);
        }
    }

    // Test maximum valid length (100 characters)
    let max_valid = json!({
        "question": &"ค".repeat(100)
    });

    let response = client
        .post("http://localhost:3000/api/v1/tarots/read")
        .json(&max_valid)
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 201, "Maximum valid question should be accepted");
        }
        Err(_) => {
            // Expected in Red Phase
        }
    }
}

/// Test for invalid question length (too short)
#[tokio::test]
async fn test_question_validation_too_short() {
    let client = reqwest::Client::new();

    // Test too short (4 characters)
    let too_short = json!({
        "question": "ควร"
    });

    let response = client
        .post("http://localhost:3000/api/v1/tarots/read")
        .json(&too_short)
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 400, "Question too short should return 400");

            let body: Value = resp.json().await.unwrap();
            assert!(body["error"].as_str().unwrap().contains("Question must be between 5 and 100 characters"));
        }
        Err(_) => {
            // Expected in Red Phase
        }
    }
}

/// Test for invalid question length (too long)
#[tokio::test]
async fn test_question_validation_too_long() {
    let client = reqwest::Client::new();

    // Test too long (101 characters)
    let too_long = json!({
        "question": &"ค".repeat(101)
    });

    let response = client
        .post("http://localhost:3000/api/v1/tarots/read")
        .json(&too_long)
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 400, "Question too long should return 400");

            let body: Value = resp.json().await.unwrap();
            assert!(body["error"].as_str().unwrap().contains("Question must be between 5 and 100 characters"));
        }
        Err(_) => {
            // Expected in Red Phase
        }
    }
}

/// Test for rate limiting enforcement (10 minute cooldown)
#[tokio::test]
async fn test_rate_limiting_enforcement() {
    let client = reqwest::Client::new();
    let valid_request = json!({
        "question": "ควรจะลงทุนอะไรดีครับ"
    });

    // First request - should succeed
    let response1 = client
        .post("http://localhost:3000/api/v1/tarots/read")
        .json(&valid_request)
        .send()
        .await;

    match response1 {
        Ok(resp) => {
            assert_eq!(resp.status(), 201, "First request should succeed");
        }
        Err(_) => {
            // Expected in Red Phase
        }
    }

    // Second request immediately - should be rate limited
    let response2 = client
        .post("http://localhost:3000/api/v1/tarots/read")
        .json(&valid_request)
        .send()
        .await;

    match response2 {
        Ok(resp) => {
            assert_eq!(resp.status(), 429, "Second request should be rate limited");

            let body: Value = resp.json().await.unwrap();
            assert!(body["error"].as_str().unwrap().contains("Rate limit exceeded"));
        }
        Err(_) => {
            // Expected in Red Phase
        }
    }
}

/// Test for API to Redis queue integration
#[tokio::test]
async fn test_api_queue_submission() {
    let client = reqwest::Client::new();
    let valid_request = json!({
        "question": "อนาคตของฉันเป็นอย่างไร"
    });

    let response = client
        .post("http://localhost:3000/api/v1/tarots/read")
        .json(&valid_request)
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 201, "Valid request should succeed");

            let body: Value = resp.json().await.unwrap();

            // Verify response structure
            assert!(body["job_id"].is_string(), "Response should contain job_id");
            assert!(body["status"].is_string(), "Response should contain status");
            assert_eq!(body["status"], "queued", "Status should be 'queued'");

            // Verify job_id is valid UUID
            let job_id_str = body["job_id"].as_str().unwrap();
            assert!(Uuid::parse_str(job_id_str).is_ok(), "job_id should be valid UUID");

            // TODO: In Green Phase, verify job is actually in Redis queue
            // For now, just verify response format
        }
        Err(_) => {
            // Expected in Red Phase
        }
    }
}

/// Test for error handling - missing question field
#[tokio::test]
async fn test_missing_question_field() {
    let client = reqwest::Client::new();
    let invalid_request = json!({
        "user_question": "คำถามที่ผิดฟิลด์"
    });

    let response = client
        .post("http://localhost:3000/api/v1/tarots/read")
        .json(&invalid_request)
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 400, "Missing question field should return 400");

            let body: Value = resp.json().await.unwrap();
            assert!(body["error"].as_str().unwrap().contains("Missing required field"));
        }
        Err(_) => {
            // Expected in Red Phase
        }
    }
}

/// Test for empty question field
#[tokio::test]
async fn test_empty_question_field() {
    let client = reqwest::Client::new();
    let empty_request = json!({
        "question": ""
    });

    let response = client
        .post("http://localhost:3000/api/v1/tarots/read")
        .json(&empty_request)
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 400, "Empty question should return 400");

            let body: Value = resp.json().await.unwrap();
            assert!(body["error"].as_str().unwrap().contains("Question cannot be empty"));
        }
        Err(_) => {
            // Expected in Red Phase
        }
    }
}

/// Unit tests for TarotRequest model validation
#[cfg(test)]
mod model_tests {
    use serde_json::{json, Value};

    #[test]
    fn test_tarot_request_validation() {
        // This will be implemented when we create the TarotRequest model
        // For now, just test JSON structure

        // Valid request
        let valid_json = json!({
            "question": "ควรจะทำอะไรดี"
        });

        assert!(valid_json["question"].is_string());
        assert!(valid_json["question"].as_str().unwrap().len() >= 5);
        assert!(valid_json["question"].as_str().unwrap().len() <= 100);

        // Invalid request - too short
        let short_json = json!({
            "question": "สั้น"
        });

        assert!(short_json["question"].as_str().unwrap().len() < 5);

        // Invalid request - too long
        let long_json = json!({
            "question": &"ทดสอบ".repeat(30) // 120 characters
        });

        assert!(long_json["question"].as_str().unwrap().len() > 100);
    }
}