-- Migration: Create Reading Jobs Schema
-- Purpose: Database schema for tarot reading jobs with state management
-- Version: 1.0
-- Author: TDD Implementation for Task #53

-- Create job_status enum first
CREATE TYPE job_status AS ENUM (
    'queued',       -- Waiting in queue
    'processing',   -- Being processed
    'succeeded',    -- Completed successfully
    'failed',       -- Failed (retry pending)
    'dlq'           -- Dead Letter Queue (permanent failure)
);

-- Create jobs table for async tarot reading processing
CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type VARCHAR(50) NOT NULL DEFAULT 'tarot_reading',

    -- Job Metadata
    schema_version VARCHAR(20) NOT NULL DEFAULT '1',
    prompt_version VARCHAR(50),
    dedupe_key VARCHAR(255) UNIQUE,              -- For idempotency

    -- Job State
    status job_status NOT NULL DEFAULT 'queued',
    payload JSONB NOT NULL,                      -- Job input data
    result JSONB,                                -- Job result (when succeeded)

    -- Processing Tracking
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 5,
    visibility_timeout_secs INTEGER DEFAULT 60,
    worker_id VARCHAR(255),

    -- Error Tracking
    last_error TEXT,
    last_error_at TIMESTAMP WITH TIME ZONE,

    -- Retry Scheduling
    next_retry_at TIMESTAMP WITH TIME ZONE,

    -- Audit & Timing
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,

    -- Constraints
    CHECK (attempts >= 0),
    CHECK (attempts <= max_attempts),
    CHECK (max_attempts > 0),
    CHECK (visibility_timeout_secs > 0)
);

-- Create indexes for optimal job queue performance
CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_created_at ON jobs(created_at DESC);
CREATE INDEX idx_jobs_status_created ON jobs(status, created_at DESC);
CREATE INDEX idx_jobs_dedupe_key ON jobs(dedupe_key);
CREATE INDEX idx_jobs_worker_id ON jobs(worker_id);
CREATE INDEX idx_jobs_retry_at ON jobs(next_retry_at) WHERE status = 'failed';

-- Create index for job type queries
CREATE INDEX idx_jobs_type ON jobs(job_type);

-- Enable Row Level Security for multi-tenant isolation
ALTER TABLE jobs ENABLE ROW LEVEL SECURITY;

-- Create RLS policy (basic - will be enhanced with user_id later)
CREATE POLICY "jobs_isolated_by_worker" ON jobs
    FOR ALL
    USING (
        -- Allow access based on worker_id or system access
        worker_id = current_setting('app.current_worker_id', true) OR
        current_setting('app.system_access', true)::boolean
    );

-- Create function to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_jobs_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to automatically update updated_at
CREATE TRIGGER trigger_jobs_updated_at
    BEFORE UPDATE ON jobs
    FOR EACH ROW
    EXECUTE FUNCTION update_jobs_updated_at();

-- Create function for safe job status transitions
CREATE OR REPLACE FUNCTION transition_job_status(
    job_id UUID,
    new_status job_status,
    p_worker_id VARCHAR DEFAULT NULL
)
RETURNS BOOLEAN AS $$
DECLARE
    current_status job_status;
    current_attempts INTEGER;
    current_max_attempts INTEGER;
BEGIN
    -- Get current job state
    SELECT status, attempts, max_attempts
    INTO current_status, current_attempts, current_max_attempts
    FROM jobs
    WHERE id = job_id;

    -- Check if job exists
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Job not found: %', job_id;
        RETURN FALSE;
    END IF;

    -- Validate status transitions
    CASE current_status
        WHEN 'queued' THEN
            IF new_status NOT IN ('processing', 'failed') THEN
                RAISE EXCEPTION 'Invalid transition from queued to %', new_status;
                RETURN FALSE;
            END IF;

        WHEN 'processing' THEN
            IF new_status NOT IN ('succeeded', 'failed', 'dlq') THEN
                RAISE EXCEPTION 'Invalid transition from processing to %', new_status;
                RETURN FALSE;
            END IF;

        WHEN 'failed' THEN
            IF new_status NOT IN ('processing', 'dlq') THEN
                RAISE EXCEPTION 'Invalid transition from failed to %', new_status;
                RETURN FALSE;
            END IF;

            -- Check if we've exceeded max attempts for dlq
            IF new_status = 'dlq' AND current_attempts < current_max_attempts THEN
                RAISE EXCEPTION 'Cannot move to DLQ before max attempts exceeded';
                RETURN FALSE;
            END IF;

        WHEN 'succeeded' THEN
            RAISE EXCEPTION 'Cannot transition from succeeded status';
            RETURN FALSE;

        WHEN 'dlq' THEN
            RAISE EXCEPTION 'Cannot transition from DLQ status';
            RETURN FALSE;

        ELSE
            RAISE EXCEPTION 'Unknown current status: %', current_status;
            RETURN FALSE;
    END CASE;

    -- Update job status and metadata
    UPDATE jobs SET
        status = new_status,
        worker_id = COALESCE(p_worker_id, worker_id),
        attempts = CASE WHEN new_status = 'processing' THEN attempts + 1 ELSE attempts END,
        started_at = CASE WHEN new_status = 'processing' AND started_at IS NULL
                          THEN CURRENT_TIMESTAMP
                          ELSE started_at END,
        completed_at = CASE WHEN new_status IN ('succeeded', 'dlq')
                           THEN CURRENT_TIMESTAMP
                           ELSE completed_at END,
        next_retry_at = CASE WHEN new_status = 'failed'
                             THEN CURRENT_TIMESTAMP + (attempts + 1 || ' seconds')::INTERVAL
                             ELSE NULL END,
        last_error_at = CASE WHEN new_status = 'failed' THEN CURRENT_TIMESTAMP ELSE NULL END
    WHERE id = job_id;

    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- Create function to create new job with dedupe check
CREATE OR REPLACE FUNCTION create_job_with_dedupe(
    p_job_type VARCHAR DEFAULT 'tarot_reading',
    p_payload JSONB,
    p_dedupe_key VARCHAR DEFAULT NULL,
    p_max_attempts INTEGER DEFAULT 5
)
RETURNS UUID AS $$
DECLARE
    new_job_id UUID;
    existing_job_id UUID;
BEGIN
    -- Check for dedupe collision
    IF p_dedupe_key IS NOT NULL THEN
        SELECT id INTO existing_job_id
        FROM jobs
        WHERE dedupe_key = p_dedupe_key
        AND status NOT IN ('succeeded', 'dlq');

        IF existing_job_id IS NOT NULL THEN
            RETURN existing_job_id; -- Return existing job ID
        END IF;
    END IF;

    -- Create new job
    INSERT INTO jobs (
        job_type,
        payload,
        dedupe_key,
        max_attempts
    ) VALUES (
        p_job_type,
        p_payload,
        p_dedupe_key,
        p_max_attempts
    ) RETURNING id INTO new_job_id;

    RETURN new_job_id;
END;
$$ LANGUAGE plpgsql;

-- Add comments for documentation
COMMENT ON TABLE jobs IS 'Async job queue for tarot reading processing';
COMMENT ON COLUMN jobs.id IS 'Unique job identifier';
COMMENT ON COLUMN jobs.job_type IS 'Type of job (tarot_reading, etc.)';
COMMENT ON COLUMN jobs.status IS 'Current job status using job_status enum';
COMMENT ON COLUMN jobs.payload IS 'Job input data as JSONB';
COMMENT ON COLUMN jobs.result IS 'Job output data as JSONB when completed';
COMMENT ON COLUMN jobs.dedupe_key IS 'Optional key for idempotent job creation';
COMMENT ON COLUMN jobs.worker_id IS 'ID of worker processing this job';
COMMENT ON COLUMN jobs.attempts IS 'Number of processing attempts made';
COMMENT ON COLUMN jobs.max_attempts IS 'Maximum allowed processing attempts';
COMMENT ON COLUMN jobs.next_retry_at IS 'When to retry failed job';
COMMENT ON COLUMN jobs.last_error IS 'Last error message from failed processing';

-- Migration completed successfully
-- Jobs table is ready for tarot reading async processing