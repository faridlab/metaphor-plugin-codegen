-- Migration: Create {{MODULE_NAME}} audit table
-- Description: Creates audit trail table for tracking all changes
-- Version: 003
-- Created: {{CURRENT_TIMESTAMP}}

-- Create {{MODULE_NAME}}_audit table for comprehensive audit trail
CREATE TABLE {{MODULE_NAME}}_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id UUID NOT NULL,
    operation VARCHAR(20) NOT NULL,
    old_data JSONB,
    new_data JSONB,
    changed_fields TEXT[],
    user_id UUID,
    user_email VARCHAR(255),
    ip_address INET,
    user_agent TEXT,
    request_id VARCHAR(100),
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',

    -- Constraints
    CONSTRAINT {{MODULE_NAME}}_audit_operation_check CHECK (
        operation IN ('INSERT', 'UPDATE', 'DELETE', 'SOFT_DELETE', 'RESTORE')
    ),
    CONSTRAINT {{MODULE_NAME}}_audit_user_email_format CHECK (
        user_email IS NULL OR user_email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'
    )
);

-- Create indexes for audit queries
CREATE INDEX idx_{{MODULE_NAME}}_audit_entity_id ON {{MODULE_NAME}}_audit(entity_id);
CREATE INDEX idx_{{MODULE_NAME}}_audit_operation ON {{MODULE_NAME}}_audit(operation);
CREATE INDEX idx_{{MODULE_NAME}}_audit_timestamp ON {{MODULE_NAME}}_audit(timestamp);
CREATE INDEX idx_{{MODULE_NAME}}_audit_user_id ON {{MODULE_NAME}}_audit(user_id);
CREATE INDEX idx_{{MODULE_NAME}}_audit_user_email ON {{MODULE_NAME}}_audit(user_email);

-- Composite indexes for common audit queries
CREATE INDEX idx_{{MODULE_NAME}}_audit_entity_timestamp ON {{MODULE_NAME}}_audit(entity_id, timestamp DESC);
CREATE INDEX idx_{{MODULE_NAME}}_audit_user_timestamp ON {{MODULE_NAME}}_audit(user_id, timestamp DESC);
CREATE INDEX idx_{{MODULE_NAME}}_audit_operation_timestamp ON {{MODULE_NAME}}_audit(operation, timestamp DESC);

-- GIN index for changed fields array
CREATE INDEX idx_{{MODULE_NAME}}_audit_changed_fields_gin ON {{MODULE_NAME}}_audit USING gin(changed_fields);

-- Add comments
COMMENT ON TABLE {{MODULE_NAME}}_audit IS 'Audit trail for all changes to {{MODULE_NAME}} entities';
COMMENT ON COLUMN {{MODULE_NAME}}_audit.id IS 'Unique identifier for the audit record';
COMMENT ON COLUMN {{MODULE_NAME}}_audit.entity_id IS 'ID of the entity that was changed';
COMMENT ON COLUMN {{MODULE_NAME}}_audit.operation IS 'Type of operation: INSERT, UPDATE, DELETE, SOFT_DELETE, RESTORE';
COMMENT ON COLUMN {{MODULE_NAME}}_audit.old_data IS 'Previous state of the entity (before update/delete)';
COMMENT ON COLUMN {{MODULE_NAME}}_audit.new_data IS 'New state of the entity (after insert/update)';
COMMENT ON COLUMN {{MODULE_NAME}}_audit.changed_fields IS 'List of fields that were changed';
COMMENT ON COLUMN {{MODULE_NAME}}_audit.user_id IS 'ID of the user who made the change';
COMMENT ON COLUMN {{MODULE_NAME}}_audit.user_email IS 'Email of the user who made the change';
COMMENT ON COLUMN {{MODULE_NAME}}_audit.ip_address IS 'IP address of the request';
COMMENT ON COLUMN {{MODULE_NAME}}_audit.user_agent IS 'User agent string of the request';
COMMENT ON COLUMN {{MODULE_NAME}}_audit.request_id IS 'Request ID for tracing';
COMMENT ON COLUMN {{MODULE_NAME}}_audit.timestamp IS 'Timestamp when the change occurred';
COMMENT ON COLUMN {{MODULE_NAME}}_audit.metadata IS 'Additional audit metadata';