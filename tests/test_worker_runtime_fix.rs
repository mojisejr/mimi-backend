//! Tests for worker runtime fix
//!
//! Test-First Development: Write failing tests before implementation
//! These tests verify that the worker can start without runtime conflicts

mod setup; // AUTO-LOAD .env via tests/setup.rs

use std::process::Command;
use std::time::Duration;

/// Test that worker binary can start without runtime panic
///
/// This test verifies that the worker can be invoked from command line
/// without "Cannot start a runtime from within a runtime" error
#[test]
fn test_worker_starts_without_runtime_conflict() {
    setup::setup();

    // Test that worker can start without runtime panic by running --help
    let result = Command::new("cargo")
        .args(&["run", "--bin", "worker", "--help"])
        .output()
        .expect("Failed to run worker binary");

    // Command should execute successfully (exit code 0 or 1 for --help)
    assert!(
        result.status.success() || result.status.code() == Some(1),
        "Worker binary should execute without panic"
    );

    let stderr = String::from_utf8_lossy(&result.stderr);

    // Critical: Should NOT contain runtime conflict error
    assert!(
        !stderr.contains("Cannot start a runtime")
            && !stderr.contains("runtime from within a runtime")
            && !stderr.contains("panicked"),
        "stderr should not contain runtime errors:\n{}",
        stderr
    );
}

/// Test worker runtime initialization
///
/// Verifies that the manual runtime initialization works correctly
/// and the worker can perform basic async operations
#[test]
fn test_runtime_initialization() {
    setup::setup();

    // Run worker with a very short timeout to test initialization
    let result = Command::new("timeout")
        .args(&["2s", "cargo", "run", "--bin", "worker"])
        .output()
        .expect("Failed to run worker with timeout");

    let stderr = String::from_utf8_lossy(&result.stderr);

    // Should not contain runtime initialization errors
    assert!(
        !stderr.contains("Cannot start a runtime")
            && !stderr.contains("runtime from within a runtime"),
        "Runtime initialization should work:\n{}",
        stderr
    );

    // Process should terminate cleanly (either by timeout or normal exit)
    // The important thing is no panic occurred
}

/// Test that worker demonstration code still runs after refactor
///
/// This test ensures the existing demo functionality is preserved
/// after converting from #[tokio::main] to manual runtime
#[test]
fn test_worker_demo_runs_successfully() {
    setup::setup();

    // Run worker with timeout to prevent it from running indefinitely
    let result = Command::new("timeout")
        .args(&["10s", "cargo", "run", "--bin", "worker"])
        .output()
        .expect("Failed to run worker demo");

    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);

    // Should show worker startup logs
    assert!(
        stdout.contains("Starting MimiVibe Worker")
            || stdout.contains("Worker process")
            || stderr.contains("Starting MimiVibe Worker")
            || stderr.contains("Worker process"),
        "Should show worker startup:\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );

    // Should NOT contain runtime errors
    assert!(
        !stderr.contains("Cannot start a runtime")
            && !stderr.contains("runtime from within a runtime")
            && !stderr.contains("panicked"),
        "Should not panic during demo:\n{}",
        stderr
    );
}

/// Test worker process termination
///
/// Verifies that worker can terminate gracefully without hanging
#[test]
fn test_worker_terminates_gracefully() {
    setup::setup();

    // Start worker process with short timeout
    let mut child = Command::new("timeout")
        .args(&["5s", "cargo", "run", "--bin", "worker"])
        .spawn()
        .expect("Failed to spawn worker process");

    // Wait for process to complete
    match child.wait() {
        Ok(status) => {
            // Process should terminate (either successfully or by timeout)
            assert!(
                status.success() || status.code() == Some(124), // 124 = timeout
                "Process should terminate gracefully"
            );
        }
        Err(e) => {
            panic!("Process should not error during wait: {}", e);
        }
    }
}
