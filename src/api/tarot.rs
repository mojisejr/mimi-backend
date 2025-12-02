//! Tarot Reading API Routes
//!
//! Endpoints for submitting tarot reading requests and retrieving results.

use crate::{
    models::{ErrorResponse, HealthResponse, TarotRequest, TarotResponse},
    queue::types::JobPayload,
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use redis::AsyncCommands;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

/// API state containing shared resources
#[derive(Clone)]
pub struct ApiState {
    pub redis_client: redis::Client,
    pub queue: Arc<dyn crate::queue::Queue + Send + Sync>,
}

/// Request a tarot reading
///
/// POST /api/v1/tarots/read
///
/// Request body:
/// ```json
/// {
///   "question": "ควรจะลงทุนอะไรดีครับ",
///   "user_id": "optional-user-id",
/// }
/// ```
///
/// Response:
/// ```json
/// {
///   "job_id": "550e8400-e29b-41d4-a716-446655440000",
///   "status": "queued",
///   "message": "Tarot reading request submitted successfully",
///   "created_at": "2023-12-02T10:30:00Z",
///   "estimated_wait_seconds": 60
/// }
/// ```
pub async fn request_reading(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<TarotRequest>,
) -> Result<Json<TarotResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Step 1: Validate the request
    if let Err(validation_error) = request.validate() {
        let error_response = ErrorResponse::new(validation_error.to_string());
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    // Step 2: Create job payload
    let job_id = Uuid::new_v4();
    let job_payload = JobPayload {
        job_id: job_id.to_string(),
        user_id: request
            .user_id
            .as_ref()
            .and_then(|u| Uuid::parse_str(u).ok())
            .unwrap_or_else(Uuid::new_v4),
        question: request.get_trimmed_question(),
        card_count: 3, // Default to 3 cards (system will randomize actual count)
        schema_version: "1".to_string(),
        prompt_version: "v2025-11-20-a".to_string(),
        dedupe_key: None,
        trace_id: Some(job_id.to_string()),
        created_at: chrono::Utc::now(),
        metadata: json!({
            "client_ip": extract_client_ip(&headers),
            "user_agent": extract_user_agent(&headers),
            "original_user_id": request.user_id
        }),
    };

    // Step 3: Submit to Redis queue
    let queue_result = match submit_to_queue(&state.queue, &job_payload).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Failed to submit job to queue: {}", e);
            let error_response = ErrorResponse::with_code(
                "Failed to submit tarot reading request to queue".to_string(),
                "QUEUE_ERROR".to_string(),
            );
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    if !queue_result {
        let error_response = ErrorResponse::with_code(
            "Queue is currently unavailable".to_string(),
            "QUEUE_UNAVAILABLE".to_string(),
        );
        return Err((StatusCode::SERVICE_UNAVAILABLE, Json(error_response)));
    }

    // Step 4: Return successful response
    let response = TarotResponse::queued(job_id);
    Ok(Json(response))
}

/// Get a previous tarot reading result
///
/// GET /api/v1/tarots/{job_id}
///
/// Response:
/// ```json
/// {
///   "job_id": "550e8400-e29b-41d4-a716-446655440000",
///   "status": "completed",
///   "result": { ... },
///   "created_at": "2023-12-02T10:30:00Z",
///   "completed_at": "2023-12-02T10:31:00Z"
/// }
/// ```
pub async fn get_reading(
    State(state): State<ApiState>,
    axum::extract::Path(job_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // Try to get job result from Redis cache
    match get_job_result(&state.redis_client, &job_id).await {
        Some(result) => Ok(Json(result)),
        None => {
            let error_response = ErrorResponse::with_code(
                "Job not found or not completed".to_string(),
                "JOB_NOT_FOUND".to_string(),
            );
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}

/// Health check endpoint
///
/// GET /api/v1/health
///
/// Response:
/// ```json
/// {
///   "status": "healthy",
///   "service": "mimivibe-backend-api",
///   "timestamp": "2023-12-02T10:30:00Z",
///   "version": "0.1.0"
/// }
/// ```
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse::healthy())
}

/// Submit job to queue
async fn submit_to_queue(
    queue: &Arc<dyn crate::queue::Queue + Send + Sync>,
    job_payload: &JobPayload,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    // Submit to queue using the queue implementation
    match queue.enqueue(job_payload.clone()).await {
        Ok(job_id) => Ok(!job_id.is_empty()),
        Err(e) => {
            // Convert Box<dyn Error> to Box<dyn Error + Send + Sync>
            let error_msg = format!("Queue enqueue error: {}", e);
            Err(
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, error_msg))
                    as Box<dyn std::error::Error + Send + Sync>,
            )
        }
    }
}

/// Get job result from Redis cache
async fn get_job_result(redis_client: &redis::Client, job_id: &Uuid) -> Option<serde_json::Value> {
    let mut conn = redis_client.get_multiplexed_async_connection().await.ok()?;

    // Look for result in Redis with key pattern: "job_result:{job_id}"
    let result_key = format!("job_result:{}", job_id);
    let result_json: Option<String> = conn.get(&result_key).await.ok()?;

    match result_json {
        Some(json_str) => serde_json::from_str(&json_str).ok(),
        None => None,
    }
}

/// Extract client IP from headers
fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    // Try various headers for client IP
    const IP_HEADERS: [&str; 8] = [
        "x-forwarded-for",
        "x-real-ip",
        "cf-connecting-ip",
        "x-client-ip",
        "x-forwarded",
        "forwarded-for",
        "forwarded",
        "x-cluster-client-ip",
    ];

    for header_name in &IP_HEADERS {
        if let Some(header_value) = headers.get(*header_name) {
            if let Ok(ip_str) = header_value.to_str() {
                // X-Forwarded-For can contain multiple IPs, take the first one
                let ip = ip_str.split(',').next().unwrap_or("").trim();
                if !ip.is_empty() {
                    return Some(ip.to_string());
                }
            }
        }
    }

    None
}

/// Extract User-Agent from headers
fn extract_user_agent(headers: &HeaderMap) -> String {
    headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}
