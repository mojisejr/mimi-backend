//! Simple Infrastructure Validation Tests
//!
//! Basic tests to validate that the test infrastructure is working correctly
//! and that the agents can be initialized for integration testing.

use std::sync::Arc;

// Simple test to verify imports work
#[test]
fn test_imports_work() {
    // This test just validates that our imports are working
    assert!(true, "Imports should work correctly");
}

#[test]
fn test_test_helper_functions() {
    // Test our helper functions work
    let cards = crate::create_test_cards(3);
    assert_eq!(cards.len(), 3, "Should create 3 test cards");

    // Test Thai text detection
    let thai_text = "สวัสดีครับ";
    assert!(crate::contains_thai_text(thai_text), "Should detect Thai text");

    let english_text = "Hello";
    assert!(!crate::contains_thai_text(english_text), "Should not detect Thai in English text");
}

#[test]
fn test_scenarios_structure() {
    let scenarios = crate::get_test_scenarios();
    assert!(!scenarios.is_empty(), "Should have test scenarios");

    for scenario in scenarios {
        assert!(!scenario.name.is_empty(), "Scenario name should not be empty");
        assert!(!scenario.question.is_empty(), "Scenario question should not be empty");
        assert!(scenario.card_count > 0, "Card count should be positive");
    }
}