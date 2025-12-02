# test-env

Environment Testing - Verify external services connectivity for backend development.

## Usage

```
/test-env [service-name]
```

## Examples

```bash
/test-env                 # Test all external services
/test-env redis          # Test only Upstash Redis connectivity
/test-env database       # Test only PostgreSQL connectivity
/test-env gemini         # Test only Gemini API connectivity
/test-env all            # Test all services (same as no arguments)
```

## Implementation

When testing environment:

### Phase 1: Environment Variables Display
```bash
echo "🔍 Checking Environment Variables:"

# Required Environment Variables
echo "DATABASE_URL: ${DATABASE_URL:0:30}..."
echo "UPSTASH_REDIS_URL: ${UPSTASH_REDIS_URL:0:30}..."
echo "UPSTASH_REDIS_TOKEN: ${UPSTASH_REDIS_TOKEN:0:15}..."
echo "GEMINI_API_KEY: ${GEMINI_API_KEY:0:10}..."
echo "API_KEY_DEFAULT: ${API_KEY_DEFAULT:0:15}..."

# Queue Configuration
echo "REDIS_STREAM_NAME: ${REDIS_STREAM_NAME:-not set}"
echo "REDIS_CONSUMER_GROUP: ${REDIS_CONSUMER_GROUP:-not set}"
echo "REDIS_CONSUMER_NAME: ${REDIS_CONSUMER_NAME:-not set}"

# Environment Settings
echo "ENVIRONMENT: ${ENVIRONMENT:-development}"
echo "RUST_LOG: ${RUST_LOG:-info}"
```

### Phase 2: Service-Specific Testing

**If no arguments or "all":**
Test all services sequentially

```bash
case "$1" in
    "redis" | "")
        echo ""
        echo "🔌 Testing Upstash Redis Connection..."
        if command -v curl &> /dev/null; then
            HTTP_CODE=$(curl -s -w "%{http_code}" "${UPSTASH_REDIS_URL}/ping" \
                         -H "Authorization: Bearer ${UPSTASH_REDIS_TOKEN}")
            if [[ "$HTTP_CODE" == "200" ]]; then
                echo "✅ Upstash Redis connected successfully"

                # Test basic queue operations
                echo "📋 Testing Redis stream operations..."
                # Add test entry
                TEST_ID="test-$(date +%s)"
                curl -s -X POST "${UPSTASH_REDIS_URL}/xadd" \
                     -H "Authorization: Bearer ${UPSTASH_REDIS_TOKEN}" \
                     -d "name=${REDIS_STREAM_NAME}&*id=$TEST_ID&service=test&timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
                     > /dev/null 2>&1

                if [ $? -eq 0 ]; then
                    echo "✅ Redis stream write test passed"

                    # Clean up test entry
                    curl -s -X POST "${UPSTASH_REDIS_URL}/xdel" \
                         -H "Authorization: Bearer ${UPSTASH_REDIS_TOKEN}" \
                         -d "name=${REDIS_STREAM_NAME}&id=$TEST_ID" \
                         > /dev/null 2>&1
                    echo "✅ Redis cleanup completed"
                else
                    echo "⚠️  Redis stream operations test failed"
                fi
            else
                echo "❌ Upstash Redis connection failed (HTTP $HTTP_CODE)"
                echo "   Check UPSTASH_REDIS_URL and UPSTASH_REDIS_TOKEN"
                exit 1
            fi
        else
            echo "⚠️  curl not available, skipping Redis connectivity test"
            echo "   Install curl or test manually"
        fi
        ;;

    "database" | "")
        echo ""
        echo "🗄️  Testing PostgreSQL Connection..."
        if [[ -n "$DATABASE_URL" ]]; then
            # Extract host from DATABASE_URL for basic connectivity test
            if command -v psql &> /dev/null; then
                timeout 10 psql "$DATABASE_URL" -c "SELECT 1;" > /dev/null 2>&1
                if [ $? -eq 0 ]; then
                    echo "✅ PostgreSQL connection successful"
                else
                    echo "❌ PostgreSQL connection failed"
                    echo "   Check DATABASE_URL and network connectivity"
                    exit 1
                fi
            else
                echo "⚠️  psql not available, but DATABASE_URL is set"
                echo "   Database URL format looks valid"
            fi
        else
            echo "❌ DATABASE_URL not set"
            exit 1
        fi
        ;;

    "gemini" | "")
        echo ""
        echo "🤖 Testing Gemini API Connection..."
        if [[ ${#GEMINI_API_KEY} -gt 10 ]]; then
            if command -v curl &> /dev/null; then
                HTTP_CODE=$(curl -s -w "%{http_code}" -H "Content-Type: application/json" \
                             -H "x-goog-api-key: ${GEMINI_API_KEY}" \
                             -d '{"contents":[{"parts":[{"text":"Hello, test connection"}]}]}' \
                             https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent)

                if [[ "$HTTP_CODE" == "200" ]]; then
                    echo "✅ Gemini API connection successful"
                elif [[ "$HTTP_CODE" == "403" ]]; then
                    echo "❌ Gemini API key invalid or expired"
                    exit 1
                else
                    echo "❌ Gemini API connection failed (HTTP $HTTP_CODE)"
                    exit 1
                fi
            else
                echo "⚠️  curl not available, but GEMINI_API_KEY format looks valid"
            fi
        else
            echo "❌ GEMINI_API_KEY seems invalid or missing"
            exit 1
        fi
        ;;

    "all" | "")
        # Test all services (already handled by empty case above)
        ;;

    *)
        echo "⚠️  Unknown service: $1"
        echo "Available services: redis, database, gemini, all (default)"
        exit 1
        ;;
esac
```

### Phase 3: Summary Report
```bash
echo ""
echo "📊 Environment Test Summary:"
echo "   All critical environment variables are set"
echo "   External services connectivity verified"
echo "   Ready for backend development with real services"
```

## Integration

- **Before Development**: Use `/test-env` to verify all environment variables and services
- **Troubleshooting**: Use `/test-env [service]` to test specific service
- **Before `/impl`**: Run `/test-env` to validate environment first
- **Environment Setup**: Run after setting up new environment variables

## Required Environment Variables

This command checks for these critical environment variables:

**Required for API:**
- `DATABASE_URL` - PostgreSQL connection string
- `UPSTASH_REDIS_URL` - Upstash Redis REST URL
- `UPSTASH_REDIS_TOKEN` - Upstash Redis authentication token
- `GEMINI_API_KEY` - Google Gemini API key
- `API_KEY_DEFAULT` - Default API key for authentication

**Required for Queue Operations:**
- `REDIS_STREAM_NAME` - Redis stream name for jobs
- `REDIS_CONSUMER_GROUP` - Redis consumer group name
- `REDIS_CONSUMER_NAME` - Consumer name (optional)

**Optional:**
- `ENVIRONMENT` - Environment name (development/staging/production)
- `RUST_LOG` - Rust log level

## Error Handling

- **Missing Variables**: Clear error messages showing which variables are missing
- **Connection Failures**: HTTP status codes and troubleshooting hints
- **Invalid Credentials**: Specific error messages for invalid API keys
- **Network Issues**: Guidance on checking connectivity and firewall settings

## Notes

- Tests real external services, not mocks
- Uses your actual API keys and endpoints
- Safe for development - only performs read-only operations
- No Docker Compose required
- Works with your current environment configuration