# MimiVibe Backend

Backend system for Tarot reading application built with Rust.

## Overview

MimiVibe Backend is a Rust-based API service that provides tarot reading functionality using Google Gemini LLM. It features a LangGraph-style agent pipeline for question filtering, analysis, and tarot reading generation.

## Features

- **Tarot Reading System**: 3-card and 5-card tarot readings with Thai language support
- **Agent Pipeline**: LangGraph-style workflow with question filtering, analysis, and reading agents
- **Google Gemini Integration**: Advanced LLM-powered tarot interpretations
- **Simple Authentication**: Environment-based API key system
- **Structured Responses**: JSON output with detailed tarot card information

## Quick Start

### Prerequisites

- Rust 1.70+
- Environment variables configuration

### Environment Setup

Create `.env.local` file:

```bash
# API Keys for different environments
TTRT_DEV_API_KEY=your_dev_api_key_here
TTRT_STAGING_API_KEY=your_staging_api_key_here
TTRT_PROD_API_KEY=your_production_api_key_here

# Google Gemini API
GEMINI_API_KEY=your_gemini_api_key_here

# Environment
APP_ENV=development  # development | staging | production
```

### Build and Run

```bash
# Build the project
cargo build --release

# Run in development
cargo run

# Run with specific environment
APP_ENV=production cargo run
```

## API Usage

### Authentication

Include API key in request headers:

```
Authorization: Bearer <your_api_key>
```

### Tarot Reading Endpoint

```
POST /api/v1/tarot/read
Content-Type: application/json
Authorization: Bearer <api_key>

{
  "question": "ความรักของฉันในช่วงนี้จะเป็นอย่างไร"
}
```

### Response Format

```json
{
  "success": true,
  "data": {
    "original_question": "ความรักของฉันในช่วงนี้จะเป็นอย่างไร",
    "question_analysis": {
      "category": "ความรัก",
      "emotion": "กังวล",
      "timeframe": "ช่วงนี้",
      "themes": ["ความสัมพันธ์", "อนาคต"]
    },
    "tarot_reading": {
      "header": "คำทักทายและทวนคำถามพร้อมเปิดประเด็นชวนคิด",
      "cards_reading": [
        {
          "id": 1,
          "name": "The Lovers",
          "displayName": "ไพ่คู่รัก",
          "imageUrl": "the_lovers.png",
          "position": 1,
          "shortMeaning": "ความรัก การเลือก ความสมดุล",
          "keywords": "ความรัก, ความสัมพันธ์, การเลือก, ความสมดุล"
        }
      ],
      "reading": "คำทำนายหลักจากไพ่ทั้งหมดในภาพรวม...",
      "suggestions": ["คำแนะนำที่ 1", "คำแนะนำที่ 2"],
      "next_questions": ["คำถามแนะนำที่ 1", "คำถามแนะนำที่ 2", "คำถามแนะนำที่ 3"],
      "final": ["สรุปผลลัพธ์ในประโยคเดียว", "คำอธิบายเพิ่มเติม"],
      "end": "ข้อความปิดท้ายอย่างอบอุ่น",
      "notice": "ข้อความเตือนให้ใช้วิจารณญาณ"
    }
  }
}
```

## Architecture

### Agent Pipeline

```
User Question → Question Filter → Question Analyzer → Random Card Count → Reading Agent → JSON Response
```

1. **Question Filter**: Validates question appropriateness for tarot reading
2. **Question Analyzer**: Analyzes context, emotions, and timeframes
3. **Random Card Count**: Generates 3 or 5 cards randomly
4. **Reading Agent**: Creates detailed tarot interpretation with Thai language support

### Technology Stack

- **Language**: Rust
- **LLM**: Google Gemini API
- **Authentication**: Simple API key validation
- **Architecture**: LangGraph-inspired agent pipeline

## Project Structure

```
mimivibe-backend/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── agents/          # Agent implementations
│   ├── api/             # API endpoints and routes
│   ├── auth/            # Authentication logic
│   ├── models/          # Data models and structs
│   └── utils/           # Utility functions
├── prompts/             # Agent system prompts
├── docs/               # Documentation
├── tests/              # Integration and unit tests
├── Cargo.toml
└── README.md
```

## Development

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Documentation

- [PRD](docs/PRD.md) - Product Requirements Document
- [API Documentation](docs/API.md) - Detailed API reference

## License

Private project - All rights reserved.