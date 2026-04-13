-- Migration: Create {{MODULE_NAME}} events table
-- Description: Creates event store table for domain events (event sourcing)
-- Version: 002
-- Created: {{CURRENT_TIMESTAMP}}

-- Create {{MODULE_NAME}}_events table for event sourcing
CREATE TABLE {{MODULE_NAME}}_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_id UUID NOT NULL REFERENCES {{MODULE_NAME}}(id) ON DELETE CASCADE,
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB NOT NULL,
    event_version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',

    -- Constraints
    CONSTRAINT {{MODULE_NAME}}_events_type_check CHECK (
        event_type IN (
            '{{PascalCaseModuleName}}Created',
            '{{PascalCaseModuleName}}Updated',
            '{{PascalCaseModuleName}}Deleted',
            '{{PascalCaseModuleName}}StatusChanged',
            '{{PascalCaseModuleName}}MetadataUpdated'
        )
    ),
    CONSTRAINT {{MODULE_NAME}}_events_version_positive CHECK (event_version > 0)
);

-- Create indexes for event queries
CREATE INDEX idx_{{MODULE_NAME}}_events_aggregate_id ON {{MODULE_NAME}}_events(aggregate_id);
CREATE INDEX idx_{{MODULE_NAME}}_events_type ON {{MODULE_NAME}}_events(event_type);
CREATE INDEX idx_{{MODULE_NAME}}_events_created_at ON {{MODULE_NAME}}_events(created_at);
CREATE INDEX idx_{{MODULE_NAME}}_events_aggregate_created ON {{MODULE_NAME}}_events(aggregate_id, created_at);

-- Composite index for event replay (aggregate_id + created_at for chronological replay)
CREATE INDEX idx_{{MODULE_NAME}}_events_replay ON {{MODULE_NAME}}_events(aggregate_id, created_at ASC);

-- GIN index for event data queries
CREATE INDEX idx_{{MODULE_NAME}}_events_data_gin ON {{MODULE_NAME}}_events USING gin(event_data);

-- Add comments
COMMENT ON TABLE {{MODULE_NAME}}_events IS 'Event store for {{MODULE_NAME}} domain events';
COMMENT ON COLUMN {{MODULE_NAME}}_events.id IS 'Unique identifier for the event';
COMMENT ON COLUMN {{MODULE_NAME}}_events.aggregate_id IS 'ID of the {{MODULE_NAME}} entity this event belongs to';
COMMENT ON COLUMN {{MODULE_NAME}}_events.event_type IS 'Type of domain event';
COMMENT ON COLUMN {{MODULE_NAME}}_events.event_data IS 'JSON data containing event details';
COMMENT ON COLUMN {{MODULE_NAME}}_events.event_version IS 'Version of the event schema';
COMMENT ON COLUMN {{MODULE_NAME}}_events.created_at IS 'Timestamp when the event occurred';
COMMENT ON COLUMN {{MODULE_NAME}}_events.metadata IS 'Additional event metadata (correlation IDs, etc.)';