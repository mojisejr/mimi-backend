# fcs

Context Management - Create and manage context discussions for iterative development planning.

## Usage

```
/fcs [topic-name]                    # Create new context issue (auto-detect type)
/fcs backend [topic-name]            # Force backend context
/fcs frontend [topic-name]          # Force frontend context
/fcs list                           # Show all active context issues
```

## Examples

```bash
# Auto-detection (recommended)
/fcs payment-api-endpoint          # Detects "API endpoint" → backend context
/fcs user-profile-page             # Detects "page" → frontend context
/fcs queue-processing-system       # Detects "system" → backend context
/fcs login-ui-component            # Detects "UI component" → frontend context

# Explicit type specification
/fcs backend tarot-reading-api     # Force backend context
/fcs frontend user-dashboard      # Force frontend context

# List contexts
/fcs list                          # Show all active context issues
```

## Implementation

When creating a new context issue:

1. **Validate GitHub CLI**: Ensure `gh` command is available
2. **Detect Context Type**:
   - If explicit type specified (`backend`/`frontend`), use that type
   - If auto-detect, analyze topic for keywords to determine type
   - **Backend keywords**: `api`, `endpoint`, `queue`, `service`, `worker`, `database`, `redis`, `gemini`, `integration`
   - **Frontend keywords**: `ui`, `page`, `component`, `interface`, `frontend`, `dashboard`, `form`
3. **Environment Validation (Backend Only)**:
   - Run `/test-env` to verify external services are accessible
   - Check required environment variables are set
   - Validate that external services can be connected to
4. **Reality-Grounded Context Analysis**:
   - Run codebase analysis using `.claude/utils/codebase-analyzer.js`
   - Scan `Cargo.toml` for available dependencies
   - Analyze existing components and patterns
   - Validate proposed topic against current capabilities
   - Generate "Current State vs Proposed" analysis
5. **Create GitHub Issue**:
   - Title: `[CONTEXT] {topic-name}`
   - Labels: `context`, `backend` or `frontend`
   - Body: Use `docs/ISSUE-TEMP.md` template
   - Replace placeholders: `{{TOPIC}}`, `{{DATE}}`, `{{TYPE}}`
   - **Enhanced**: Include "Codebase Reality Check" section with actual analysis
   - **Backend**: Include environment validation results
   - **Frontend**: Include component availability analysis
6. **Track context**: Add to `.claude/active_contexts` file
7. **Display results**: Show issue URL and next steps

## Context Type Detection Algorithm

### Backend Detection Patterns:
- **API keywords**: `api`, `endpoint`, `route`, `handler`, `request`, `response`
- **Queue keywords**: `queue`, `job`, `worker`, `process`, `async`, `stream`
- **Database keywords**: `database`, `schema`, `migration`, `query`, `repository`
- **Service keywords**: `service`, `integration`, `external`, `gemini`, `redis`, `supabase`
- **Infrastructure keywords**: `auth`, `security`, `monitoring`, `logging`, `metrics`

### Frontend Detection Patterns:
- **UI keywords**: `ui`, `interface`, `component`, `element`, `widget`
- **Page keywords**: `page`, `view`, `screen`, `layout`, `dashboard`
- **Interaction keywords**: `form`, `input`, `button`, `modal`, `dialog`
- **Visual keywords**: `design`, `style`, `theme`, `responsive`, `animation`
- **User Experience**: `navigation`, `menu`, `breadcrumb`, `notification`

When listing active contexts:

1. **Read tracker**: Parse `.claude/active_contexts` file
2. **Display list**: Show issue numbers and topics
3. **Provide guidance**: Suggest next actions

## Template Integration

Uses `docs/ISSUE-TEMP.md` template which contains:
- DISCUSSION LOG section for iterative updates
- ACCUMULATED CONTEXT section for key decisions
- PLANNING READINESS CHECKLIST for validation
- 🔧 BACKEND ENVIRONMENT VALIDATION section (for backend contexts)
- 🏗️ TECHNICAL ARCHITECTURE section (backend or frontend specific)

### Backend Context Specific Sections:
- **Backend Services**: APIs, workers, queue systems
- **API Endpoints**: Specific endpoints to be implemented
- **External Integrations**: Gemini API, Upstash Redis, Supabase
- **Environment Variables**: Required configuration
- **Real Service Testing**: Environment validation requirements

### Frontend Context Specific Sections:
- **Frontend Components**: UI components to be created
- **User Interface**: Page layouts, responsive design
- **User Experience**: Navigation, forms, interactions
- **Visual Design**: Styling, themes, animations

## Files

- `docs/ISSUE-TEMP.md` - Context issue template
- `.claude/active_contexts` - Tracks active context issues
- `.claude/utils/codebase-analyzer.js` - Reality analysis utilities
- GitHub Issues - Stores context discussions

## Codebase Reality Analysis

The enhanced `/fcs` command now performs actual codebase analysis with type-specific focus:

### Reality Check Process:
1. **Dependency Scan**: Checks `Cargo.toml` for installed crates
2. **Backend Analysis**: Scans `src/` for API routes, services, workers, queue systems
3. **Frontend Analysis**: Scans `src/components` for existing UI components (if applicable)
4. **Pattern Detection**: Identifies existing patterns (API handlers, auth flows, queue processors)
5. **Capability Validation**: Validates if proposed topic is realistic for detected type
6. **Gap Analysis**: Identifies missing requirements and provides installation steps

### Backend Reality Check Example:
```
## Codebase Reality Check

**Current State:**
- Framework: Rust with Axum framework
- Database: PostgreSQL via Supabase
- Queue System: Upstash Redis Streams
- External Services: Gemini API integration
- Available: Agent pipeline, queue workers, API handlers
- Missing: Payment processing, advanced monitoring

**Topic "Payment API" Analysis:**
- ✅ Realistic: Can implement using existing patterns
- ✅ Available: Axum handlers, database models, queue processing
- ❌ Missing: Payment processing library (Stripe SDK)
- 💡 Recommendation: Add stripe-rust crate or use Stripe HTTP API
- 🏗️ Implementation: Use existing API handler patterns with payment validation
```

### Frontend Reality Check Example:
```
## Codebase Reality Check

**Current State:**
- Framework: Next.js 14 App Router (if applicable)
- Database: PostgreSQL via Supabase + Prisma ORM
- Available: Basic UI components (Button, Card, Input)
- Missing: Form libraries, toast notifications, testing framework

**Topic "User Dashboard" Analysis:**
- ✅ Realistic: Can implement using existing patterns
- ✅ Available: Layout components, authentication, data fetching
- ❌ Missing: Chart libraries, advanced table components
- 💡 Recommendation: Add recharts for data visualization
- 🏗️ Implementation: Use existing Card/Dialog components and patterns
```

### Hallucination Prevention:
- ✅ All technical assumptions validated against actual code structure
- ✅ Component existence verified before referencing
- ✅ Dependencies checked before proposing features
- ✅ Environment validation for backend contexts
- ✅ Realistic recommendations based on current capabilities

## Integration

This command integrates with:
- `/test-env` - Environment validation before backend context creation
- `/plan` - Context should reach `[Ready for Planning]` status before planning
- Workflow system - Context is Phase 1 of development workflow

## Context Status Flow

1. **Environment Validation** (Backend only) - `/test-env` passes all tests
2. **Created** - Initial context issue created with type-specific labels
3. **Discussion** - Iterative updates via `/fcs [topic]`
4. **Ready for Planning** - Context ready for task creation
5. **Implementation Ready** - Context ready for implementation

## Backend Context Flow Specifics

### Pre-Creation Requirements:
- **Environment Validation**: `/test-env` must pass all connectivity tests
- **Service Access**: External services must be reachable
- **Configuration**: Required environment variables must be set

### Context Labels:
- **Backend contexts**: `context`, `backend`
- **Frontend contexts**: `context`, `frontend`
- **Auto-detected**: Based on topic analysis or explicit type

### Template Selection:
- **Backend**: Uses enhanced `docs/ISSUE-TEMP.md` with backend validation sections
- **Frontend**: Uses `docs/ISSUE-TEMP.md` with frontend-focused sections
- **Unified**: Same template file but content adapts based on context type

## Notes

- Always creates GitHub Issues (NEVER local .md files)
- Context issues are living documents for discussion
- Use existing context issue when adding to topics
- Context must be ready before creating tasks with `/plan`
- Backend contexts require environment validation before creation
- Auto-detection works for most common scenarios but explicit type is available for edge cases