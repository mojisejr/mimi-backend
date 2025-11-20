## Project Overview

**Project Name**: MimiVibe Backend

**Repository**: https://github.com/your-username/mimivibe-backend
**Author**: Your Name

**Description**: A Rust-based backend system for tarot reading application using Google Gemini LLM with LangGraph-style agent pipeline for question filtering, analysis, and detailed tarot interpretations in Thai language.

---

## ⚠️ CRITICAL SAFETY RULES

### 🚨 FORBIDDEN ACTIONS (NEVER ALLOWED)

- ❌ **NEVER merge PRs yourself** - Provide PR link and wait for user instructions
- ❌ **NEVER work on main/staging branches** - Always use feature branches
- ❌ **NEVER delete critical files** (.env, .git/, `Cargo.toml`, `Cargo.lock`, `src/`)
- ❌ **NEVER commit sensitive data** (API keys, passwords, secrets) - Use environment variables
- ❌ **NEVER skip 100% validation** (build, lint, test) - Must pass completely
- ❌ **NEVER use git push --force** - Only use --force-with-lease when absolutely necessary
- ❌ **NEVER implement without task issue** - Must use =plan command first

### 📋 MANDATORY WORKFLOW RULES

- ✅ **ALWAYS** sync main branch before any implementation: `git checkout main && git pull origin main`
- ✅ **ALWAYS** verify task issue exists: `#[issue-number]` before `=impl`
- ✅ **ALWAYS** use feature branch naming: `feature/task-[issue-number]-[description]`
- ✅ **ALWAYS** ensure 100% build success before commit: `cargo build --release`
- ✅ **ALWAYS** ensure 100% lint pass before commit: `cargo clippy -- -D warnings`
- ✅ **ALWAYS** use template-guided workflow with proper context validation
- ✅ **ALWAYS** verify code formatting: `cargo fmt -- --check` (consistent formatting)

---

## 📋 Workflow System

### Template Integration

**Context Issue Template** - `/docs/ISSUE-TEMP.md`:

- Used for: `=fcs > [topic-name]` or `=fcs > [CONTEXT]`
- **ALWAYS creates GitHub Issue** - Never creates local .md files
- Creates living document for iterative discussion
- Contains: DISCUSSION LOG, ACCUMULATED CONTEXT, PLANNING READINESS CHECKLIST

**Task Issue Template** - `/docs/TASK-ISSUE-TEMP.md`:

- Used for: `=plan > [task description]`
- **ALWAYS creates GitHub Issue** - Never creates local .md files
- Creates atomic tasks based on current mode (MANUAL/COPILOT)
- Contains: EXECUTION MODE field, 100% validation requirements

**Knowledge Issue Template** - `/docs/KNOWLEDGE-TEMP.md`:

- Used for: `=kupdate [category] "[topic]"`
- **ALWAYS creates GitHub Issue** - Never creates local .md files
- Creates structured knowledge entries with AI honest feedback
- Contains: Problem → Solution → Lessons Learned → Links

### Mode-Based Execution System

**Default Mode**: MANUAL (Claude implementation)

**Mode Commands**:

```bash
=mode manual     # Tasks assigned to Claude (non-Copilot agent)
=mode copilot     # Tasks assigned to @copilot
=mode status      # Show current execution mode
```

**Mode-Specific Behavior**:

- **MANUAL Mode**: `=plan` creates tasks assigned to Claude, `=impl` triggers Claude implementation using code editing tools
- **COPILOT Mode**: `=plan` creates tasks assigned to @copilot, `=impl` triggers copilot implementation

### Core Commands

**✅ NEW: Claude Code Slash Commands Implemented!**
All workflow commands are now available as proper Claude Code slash commands (markdown files in `.claude/commands/`).

```bash
# Mode Management
/mode [manual|copilot|status]  # Set or show execution mode

# Context Management
/fcs [topic-name]              # Create new Context GitHub Issue
/fcs list                      # Show all active Context Issues

# Task Management
/plan [task description]       # Create Task GitHub Issue using docs/TASK-ISSUE-TEMP.md
/impl [issue-number]           # Implementation workflow for specific GitHub issue
/impl [issue-number] [msg]     # Implementation with additional context
/pr [feedback]                 # Create Pull Request from feature branch (to staging)

# Knowledge Management
/khub                          # 🔍 Read Knowledge Hub #102 (MANDATORY first step)
/kupdate [category] "[topic]"  # Create Knowledge GitHub Issue (CHECK existing numbers!)
/klink [knowledge-issue-number] # Link knowledge entry to Knowledge Hub #102
/ksync                         # Synchronize Knowledge Hub with all entries
/ksearch "[query]"             # Search across all knowledge entries
/krecent                       # Show last 5 knowledge updates
/kcategory [category]          # Show knowledge for specific category

# Other Commands
/rrr [message]                 # Create daily retrospective GitHub Issue

# Legacy = Commands (still supported for backward compatibility)
=fcs > [topic-name]           # Create new Context GitHub Issue
=plan > [task description]    # Create Task GitHub Issue
=impl > [issue-number]        # Implementation workflow
=khub                         # Read Knowledge Hub #102
# ... (all other = commands still work)
```

### Claude Code Slash Command Features

- **Proper Implementation**: Markdown files in `.claude/commands/` directory
- **Claude Integration**: Processed and executed by Claude Code
- **Rich Documentation**: Each command includes comprehensive usage, examples, and implementation details
- **Error Handling**: Clear error messages and helpful suggestions
- **Validation**: Automatic checking of prerequisites and dependencies
- **Help System**: All commands support help via detailed documentation

### Command Structure

All slash commands follow this structure:
- **Usage**: Clear syntax and parameter description
- **Examples**: Practical usage scenarios
- **Implementation**: Step-by-step execution details
- **Integration**: How commands work together
- **Error Handling**: Common issues and solutions
- **Files**: Related files and dependencies

### Template-Driven Workflow Process

1. **Phase 1**: `/fcs [topic]` → Create initial context **GitHub Issue** (NEVER .md file)
2. **Phase 2**: `/fcs [topic]` → Update context **GitHub Issue** iteratively
3. **Phase 3**: Context reaches `[Ready for Planning]` status → Ready for planning
4. **Phase 4**: `/plan [task]` → Create atomic **GitHub Issues** (NEVER .md files)
5. **Phase 5**: `/impl [issue-number]` → Implement specific GitHub issue based on mode

**💡 Enhanced Workflow with Claude Code Slash Commands:**
- Use `/mode [manual|copilot]` to set execution mode
- Commands processed by Claude Code with intelligent execution
- Rich documentation and help built into each command
- Comprehensive error handling and validation
- All workflows maintain the same template-driven approach
- Legacy `=` commands remain supported for backward compatibility

### Implementation Workflow (MANDATORY)

**Pre-Implementation Checklist**:

1. **Staging Sync**: `git checkout staging && git pull origin staging`
2. **Task Verification**: Confirm Task **GitHub Issue** `#[issue-number]` exists and is [TASK] type
3. **Context Status**: Verify Context **GitHub Issue** is `[Ready for Planning]` or `[Implementation Ready]`
4. **Environment Check**: `git status` - working directory must be clean

**Implementation Steps**:

1. **Create Feature Branch**: `git checkout -b feature/task-[issue-number]-[description]`
2. **Execute Implementation**: Follow task requirements, use TodoWrite for complex tasks
3. **Build Validation**: `cargo build --release` (100% success - zero errors)
4. **Lint Validation**: `cargo clippy -- -D warnings` (100% pass - zero warnings)
5. **Format Check**: `cargo fmt -- --check` (consistent formatting)
6. **Type Check**: `cargo check` (comprehensive type checking)
7. **Run Tests**: `cargo test` (if applicable)
8. **Commit Changes**:

   ```bash
   git add .
   git commit -m "feat: [feature description]

   - Address #[issue-number]: [task title]
   - Build validation: 100% PASS (`cargo build --release`)
   - Lint validation: 100% PASS (`cargo clippy -- -D warnings`)
   - Format validation: 100% PASS (`cargo fmt -- --check`)

   🤖 Generated with Claude Code
   Co-Authored-By: Claude <noreply@anthropic.com>"
   ```

9. **Push Branch**: `git push -u origin feature/task-[issue-number]-[description]`

**Post-Implementation**:

- **MANUAL Mode**: Claude implements and pushes to feature branch, user uses `/pr` to create PR
- **COPILOT Mode**: GitHub Copilot implements and pushes to feature branch, user uses `/pr` to create PR

---

## 🧠 Knowledge Management System

### Knowledge Workflow Integration

**Knowledge Capture Points**:

- **After Implementation**: When `=impl` completes successfully, use `=kupdate` to document learnings **(auto-prompts for hub linking)**
- **After Context Discussion**: When `=fcs` reaches key decisions, use `=kupdate` to capture insights **(auto-prompts for hub linking)**
- **After Chat Discoveries**: When breakthrough solutions are found, use `=kupdate` to preserve knowledge **(auto-prompts for hub linking)**

**Enhanced Knowledge Workflow**:

1. **🔍 Pre-Creation Check**: `=khub` → Read Knowledge Hub #102 FIRST to check existing KNOW-[CATEGORY]-XXX numbers
2. **Verify**: Check category section for existing numbers to avoid duplicates (e.g., KNOW-DEVICE-001, KNOW-DEVICE-002)
3. **Create**: `=kupdate [category] "[topic]"` → Creates knowledge issue with next available number
4. **Prompt**: System asks "Link to Knowledge Hub #102? (y/n)"
5. **Link**: If "y" → Automatically runs `=klink`
6. **Sync**: Use `=ksync` to ensure hub is fully synchronized
7. **Discover**: All knowledge accessible through `=khub` navigation

### Knowledge Categories

**Standard Categories**:

- `device` - CU12, KU16, SerialPort, hardware integration
- `database` - SQLite, Sequelize, migrations, queries
- `architecture` - Design patterns, structural decisions
- `debug` - Error solutions, troubleshooting, workarounds
- `workflow` - Process improvements, automation
- `frontend` - React, Electron, UI components
- `backend` - Node.js, APIs, services

### Knowledge ID System

**Format**: `KNOW-[CATEGORY]-[NUMBER]`

- Example: `KNOW-DEVICE-001`, `KNOW-DATABASE-015`
- Auto-increment per category
- Easy reference and cross-linking

### 🔍 Knowledge ID Conflict Prevention (CRITICAL)

**MANDATORY Pre-Creation Checklist**:

1. **ALWAYS run `=khub` first** - Read Knowledge Hub #102 completely
2. **Check existing numbers** in your category section (e.g., "Device Knowledge")
3. **Identify next available number** (if 001, 002 exist, use 003)
4. **Never assume** - always verify existing entries before creating

**Common Mistakes to Avoid**:

- ❌ Creating KNOW-DEVICE-001 when it already exists
- ❌ Not checking Knowledge Hub #102 before creating entries
- ❌ Assuming numbers without verification
- ❌ Creating duplicate knowledge IDs

**Correct Workflow Example**:

```bash
# ❌ WRONG (creates duplicate)
= kupdate device "SHT30 sensor fix"  # Creates KNOW-DEVICE-001 (duplicate!)

# ✅ RIGHT (prevents duplicates)
= khub                              # Read Knowledge Hub #102
# See: KNOW-DEVICE-001, KNOW-DEVICE-002 exist
= kupdate device "SHT30 sensor fix" # Creates KNOW-DEVICE-003 (correct!)
```

### Auto-Label Creation

**System Behavior**:

```bash
# When =kupdate device "CU12 lock-back solution" is used:
# 1. Check if 'knowledge-device' label exists
# 2. If not, create: gh label create knowledge-device --color "1d76db" --description "Device integration knowledge"
# 3. Apply label to knowledge issue
# 4. Auto-generate Knowledge ID: KNOW-DEVICE-001
```

**Knowledge Labels Created Automatically**:

- `knowledge-device` - Device integration knowledge
- `knowledge-database` - Database and persistence knowledge
- `knowledge-architecture` - System design and patterns
- `knowledge-debug` - Debugging and troubleshooting
- `knowledge-workflow` - Development workflow improvements

### Enhanced Knowledge Hub Integration

**New Automated Commands**:

**`=klink [knowledge-issue-number]`**:

- Automatically detects category from knowledge issue labels
- Places knowledge link in appropriate Knowledge Hub section
- Updates statistics counters
- Maintains proper markdown formatting

**`=ksync`**:

- Scans all issues with `knowledge-*` labels
- Synchronizes Knowledge Hub with all existing knowledge entries
- Updates statistics and distribution
- Fixes broken links and formatting
- Ensures hub reflects current knowledge base state

**Enhanced `=kupdate` Workflow**:

1. Creates knowledge GitHub issue ✅
2. **Automatically prompts**: "Link to Knowledge Hub #102? (y/n)"
3. If "y": Runs `=klink` automatically ✨
4. Maintains consistency across knowledge system

**Command Implementation Details**:

**`=klink [issue-number]` Implementation**:

1. **Issue Analysis**: Extract title, labels, and description
2. **Category Detection**: Parse `knowledge-[category]` label
3. **Format Entry**: `**KNOW-[CATEGORY]-[NUMBER]**: [Title](issue-link) - Brief description`
4. **Section Insert**: Add to appropriate "Recent Entries" section
5. **Statistics Update**: Increment total and category counts
6. **Timestamp Update**: Set "Last Updated" to current date

**`=ksync` Implementation**:

1. **Knowledge Discovery**: Scan all issues with `knowledge-*` labels
2. **Category Processing**: Group by label type (device, database, etc.)
3. **Entry Generation**: Create standardized format for each found issue
4. **Hub Reconstruction**: Replace all category sections with complete lists
5. **Statistics Calculation**: Recalculate all counts from scratch
6. **Format Validation**: Ensure proper markdown structure and valid links

**Hub Integration Benefits**:

- ✅ **No more manual linking required**
- ✅ **Automatic statistics updates**
- ✅ **Consistent formatting maintained**
- ✅ **Centralized knowledge discovery**
- ✅ **Real-time hub synchronization**

### Knowledge Search & Retrieval

**Search Capabilities**:

```bash
=ksearch "CU12 lock-back"    # Full-text search across all knowledge
=kcategory device           # Show all device-related knowledge
=krecent                    # Last 5 knowledge entries
=khub                       # Go to main Knowledge Hub issue
=ksync                      # Synchronize hub with all knowledge entries
=klink 116                  # Link knowledge issue #116 to hub
```

**Search Optimization**:

- Knowledge entries include searchable tags
- Problem statements use clear, technical language
- Solutions include specific keywords and technologies
- Cross-references link related knowledge
- Hub ensures all knowledge is discoverable from central location

### Knowledge Structure

**Each Knowledge Entry Contains**:

- **Problem Statement**: Clear description of what was solved
- **Solution Implementation**: Step-by-step working solution
- **AI Honest Feedback**: What worked, what didn't, lessons learned
- **Things to Avoid**: Common pitfalls and their consequences
- **Prerequisites**: What to check before starting
- **AI Self-Improvement**: Insights for future problem-solving
- **Links & References**: Connections to source issues/PRs/code
- **Verification Status**: Testing and validation state

---

## 🏗️ Technical Architecture

### Core Stack

- **Language**: Rust
- **Web Framework**: TBD (Axum/Actix-web)
- **LLM Integration**: Google Gemini API
- **Authentication**: Environment-based API key system
- **Agent Pipeline**: LangGraph-inspired workflow engine
- **Data Storage**: In-memory storage (Phase 1)
- **Deployment**: TBD
- **Language Support**: Thai language tarot readings

### Project Structure

```
mimivibe-backend/
├── README.md                   # Project overview and quick start
├── docs/                       # Documentation and templates
├── src/                        # Rust source code
│   ├── main.rs                 # Application entry point
│   ├── lib.rs                  # Library root
│   ├── agents/                 # Agent implementations
│   │   ├── mod.rs
│   │   ├── question_filter.rs  # Question validation agent
│   │   ├── question_analyzer.rs # Question analysis agent
│   │   └── reading_agent.rs    # Tarot reading agent
│   ├── api/                    # API endpoints and routes
│   │   ├── mod.rs
│   │   └── tarot.rs           # Tarot reading endpoints
│   ├── auth/                   # Authentication logic
│   │   ├── mod.rs
│   │   └── api_key.rs         # API key validation
│   ├── models/                 # Data models and structs
│   │   ├── mod.rs
│   │   ├── request.rs         # Request models
│   │   ├── response.rs        # Response models
│   │   └── tarot.rs           # Tarot-specific models
│   └── utils/                  # Utility functions
│       ├── mod.rs
│       └── gemini.rs          # Gemini API client
├── prompts/                    # Agent system prompts
│   ├── QUESTION-FILTER.md      # Question filter prompt
│   ├── QUESTION-ANALYSIS.md    # Question analysis prompt
│   └── READING-AGENT.md        # Reading agent prompt
├── tests/                      # Integration and unit tests
├── Cargo.toml                  # Rust dependencies
└── .env.local                  # Environment variables (not committed)
```

### Key Features

- **Tarot Reading System**: 3-card and 5-card tarot readings with Thai language support
- **Agent Pipeline**: LangGraph-style workflow with sequential agent execution
- **Question Processing**: Question filtering and analysis before tarot reading
- **Google Gemini Integration**: Advanced LLM-powered tarot interpretations
- **Simple Authentication**: Environment-based API key validation
- **Structured Responses**: JSON output with detailed tarot card information
- **Thai Language Support**: Complete tarot reading workflow in Thai
- **Random Card Generation**: 3 or 5 card selection with traditional tarot deck
- **Modern Architecture**: Rust-based with agent pipeline pattern

### Development Commands

```bash
cargo run              # Run development server
cargo build            # Build the project
cargo test             # Run all tests
cargo clippy           # Lint checks
cargo fmt              # Format code
cargo check            # Type checking without building
```

### Performance Metrics

- **API Response Time**: Target < 5 seconds (including LLM processing)
- **LLM Processing Time**: < 3 seconds per reading
- **Concurrent Users**: Support 100+ simultaneous requests
- **Memory Usage**: < 512MB per instance
- **Build Time**: ~30 seconds for production build
- **Monthly Cost**: ~$10-30 (hosting + Gemini API)

---

## 🎯 Quality Standards

### Code Quality Requirements

- **Rust**: Use idiomatic Rust with strong type coverage
- **Clippy**: Zero warnings (enforced in CI with `cargo clippy -- -D warnings`)
- **Rustfmt**: Consistent code formatting (`cargo fmt` / `cargo fmt -- --check`)
- **Build**: 100% success rate before commit (`cargo build --release`)
- **Tests**: Unit and integration tests for critical paths
- **Type Safety**: Ensure `cargo check` passes
- **Error Handling**: Comprehensive error handling and domain-specific errors

### API Quality Standards

- **Response Times**: p95 < 200ms for all endpoints
- **Error Handling**: Always return structured JSON errors with status codes
- **Rate Limiting**: Enforce per-user limits via database tracking
- **Input Validation**: Validate all user inputs with zod schemas
- **Session Security**: 7-day session expiration, secure cookie management
- **HTTPS Only**: Enforced in production via Vercel
- **Row-Level Security**: Zero-trust data access via Supabase RLS

### Performance Standards

- **Startup Time**: Next.js server ready within 2-3 seconds
- **Database Queries**: < 50ms per query (with proper indexing)
- **Page Navigation**: < 200ms for client-side transitions
- **API Response**: < 200ms for all endpoints (p95)
- **Concurrent Users**: Handle 100+ concurrent connections
- **Bundle Size**: < 1MB initial JavaScript load
- **Image Optimization**: WebP format with lazy loading

### Security Standards

- **Secrets Management**: Use .env.local, never commit sensitive data
- **Database Access**: All queries use Prisma ORM with parameterization
- **Authentication**: better-auth with LINE OAuth + session management
- **CORS**: Configured for frontend domain only
- **Rate Limiting**: Per-user limits on sensitive endpoints
- **Row-Level Security**: Zero-trust data access via Supabase RLS policies
- **Input Validation**: All inputs validated via zod schemas
- **File Security**: Image uploads restricted to safe formats and sizes

### Template-Guided Quality

- **Context Issues**: Complete PLANNING READINESS CHECKLIST ✅ (Always GitHub Issues)
- **Task Issues**: 100% build/lint/test requirements mandatory (Always GitHub Issues)
- **Mode Execution**: Follow mode-specific behavior exactly
- **Template Consistency**: All issues follow template structures
- **File Policy**: NEVER create local .md files for issues - ALWAYS use GitHub Issues

---

## 📚 Reference Materials

### Templates

- `/docs/ISSUE-TEMP.md` - Context Issue Template for iterative discussions
- `/docs/TASK-ISSUE-TEMP.md` - Atomic Task Template for implementation
- `/docs/KNOWLEDGE-TEMP.md` - Knowledge Issue Template for structured learning

### Performance Metrics

- **Target**: API response time < 200ms (p95)
- **Goal**: 99.9% uptime for livestock management service
- **Reliability**: 99.99% data consistency and availability
- **Database**: PostgreSQL via Supabase with automatic scaling
- **Storage**: Supabase Storage with RLS policies
- **Cost**: ~$25-50/month for full stack at scale

### Security Notes

- **Input Validation**: Comprehensive validation for all user inputs via zod
- **Authentication**: better-auth with LINE OAuth + session management
- **Data Protection**: Encrypted connections, secure session storage
- **Access Control**: Role-based access (Owner, Staff levels)
- **Data Security**: Row-Level Security policies for zero-trust access
- **Audit Trail**: Complete logs for animal and activity management

---

_This document focuses on agent-critical information for efficient workflow execution and safe development practices._