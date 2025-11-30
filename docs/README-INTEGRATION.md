# Integration test instructions

This document explains how to run integration tests that use the real Google Gemini LLM.

Warning: these tests will call external LLM APIs and may incur cost. Only run them locally or in a controlled CI environment with proper secrets.

Environment variables required:
- GEMINI_API_KEY — set to your Google Gemini API key
- RUN_GEMINI_INTEGRATION=true — required flag so tests that call the real LLM will run

Example (macOS / zsh):

```bash
export GEMINI_API_KEY=your_real_gemini_key_here
export RUN_GEMINI_INTEGRATION=true
# Run just the integration tests marked for Gemini (example test file)
cargo test --test tarot_integration -- --ignored

# Or run all ignored tests
cargo test -- --ignored
```

Notes:
- Unit tests MUST not call external LLMs. They should mock the LLM client.
- Integration tests should be gated and only run when both environment variables are present.
