# Integration testing guide — Reading Agent / Gemini

This document describes how to run the live integration tests that call Google Gemini and how we gate tests behind environment variables so CI doesn't call external LLMs by default.

## Gated integration tests
Integration tests that call Gemini will only run automatically when the following environment variables are present and set appropriately:

- GEMINI_API_KEY — (required) the real Gemini API key
- RUN_GEMINI_INTEGRATION=true — (must be set) opt-in flag to run live Gemini tests

If either of these variables is missing or RUN_GEMINI_INTEGRATION is not 'true', the live LLM integration tests must skip.

## Running integration tests locally
1. Create a `.env.local` file at the repository root (do NOT commit it).
2. Add the following values:

```
GEMINI_API_KEY=your_real_gemini_key_here
RUN_GEMINI_INTEGRATION=true
```

3. Run the integration tests that call Gemini:

```bash
# run only integration tests marked for gemini
cargo test -- --ignored
```

> Note: Integration tests should be marked `#[ignore]` and will be conditionally executed when you intentionally run them. They must not run on CI unless the CI environment is intentionally configured and approved.

## Safety and audit
- Keep the API key in your machine or in secure CI secrets; do not commit real secrets to the repo.
- Prefer running live LLM tests manually during local development or on secure, approved CI jobs with secrets set by the ops team.
