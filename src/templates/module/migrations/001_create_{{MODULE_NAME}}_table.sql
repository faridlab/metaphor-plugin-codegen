-- Migration: Create {{MODULE_NAME}} table
-- Description: Creates the main {{MODULE_NAME}} entity table with all necessary indexes
-- Version: 001
-- Created: {{CURRENT_TIMESTAMP}}

-- Enable UUID extension if not already enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create {{MODULE_NAME}} table
CREATE TABLE {{MODULE_NAME}} (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,

    -- Constraints
    CONSTRAINT {{MODULE_NAME}}_status_check CHECK (status IN ('active', 'inactive', 'pending', 'archived')),
    CONSTRAINT {{MODULE_NAME}}_name_length CHECK (length(name) >= 2 AND length(name) <= 255)
);

-- Create indexes for performance
CREATE INDEX idx_{{MODULE_NAME}}_status ON {{MODULE_NAME}}(status);
CREATE INDEX idx_{{MODULE_NAME}}_created_at ON {{MODULE_NAME}}(created_at);
CREATE INDEX idx_{{MODULE_NAME}}_updated_at ON {{MODULE_NAME}}(updated_at);
CREATE INDEX idx_{{MODULE_NAME}}_deleted_at ON {{MODULE_NAME}}(deleted_at);
CREATE INDEX idx_{{MODULE_NAME}}_name ON {{MODULE_NAME}}(name);

-- GIN index for metadata JSONB queries
CREATE INDEX idx_{{MODULE_NAME}}_metadata_gin ON {{MODULE_NAME}} USING gin(metadata);

-- Full-text search index for name and description
CREATE INDEX idx_{{MODULE_NAME}}_search ON {{MODULE_NAME}} USING gin(
    to_tsvector('english', name || ' ' || COALESCE(description, ''))
);

-- Partial index for active records (optimization for common queries)
CREATE INDEX idx_{{MODULE_NAME}}_active ON {{MODULE_NAME}}(id) WHERE deleted_at IS NULL AND status = 'active';

-- Composite index for common list queries (status + created_at for pagination)
CREATE INDEX idx_{{MODULE_NAME}}_status_created_at ON {{MODULE_NAME}}(status, created_at DESC);

-- Add comments for documentation
COMMENT ON TABLE {{MODULE_NAME}} IS 'Main table for storing {{MODULE_NAME}} entities';
COMMENT ON COLUMN {{MODULE_NAME}}.id IS 'Unique identifier for the {{MODULE_NAME}} entity';
COMMENT ON COLUMN {{MODULE_NAME}}.name IS 'Name of the {{MODULE_NAME}} entity (2-255 characters)';
COMMENT ON COLUMN {{MODULE_NAME}}.description IS 'Optional detailed description';
COMMENT ON COLUMN {{MODULE_NAME}}.status IS 'Current status: active, inactive, pending, or archived';
COMMENT ON COLUMN {{MODULE_NAME}}.metadata IS 'Flexible JSON metadata for additional attributes';
COMMENT ON COLUMN {{MODULE_NAME}}.created_at IS 'Timestamp when the entity was created';
COMMENT ON COLUMN {{MODULE_NAME}}.updated_at IS 'Timestamp when the entity was last updated';
COMMENT ON COLUMN {{MODULE_NAME}}.deleted_at IS 'Timestamp for soft delete (null if not deleted)';

-- Create trigger for automatic updated_at timestamp
CREATE OR REPLACE FUNCTION update_{{MODULE_NAME}}_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER trigger_{{MODULE_NAME}}_updated_at
    BEFORE UPDATE ON {{MODULE_NAME}}
    FOR EACH ROW
    EXECUTE FUNCTION update_{{MODULE_NAME}}_updated_at();