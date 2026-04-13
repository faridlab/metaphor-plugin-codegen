-- Migration: {{MIGRATION_NAME}}
-- Create {{TABLE_NAME}} table

CREATE TABLE IF NOT EXISTS {{TABLE_NAME}} (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'ACTIVE',
    created_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,
    version BIGINT NOT NULL DEFAULT 1,

    CONSTRAINT {{TABLE_NAME}}_status_check CHECK (status IN ('ACTIVE', 'INACTIVE', 'SUSPENDED', 'ARCHIVED')),
    CONSTRAINT {{TABLE_NAME}}_version_check CHECK (version > 0)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_{{TABLE_NAME}}_created_at ON {{TABLE_NAME}} (created_at);
CREATE INDEX IF NOT EXISTS idx_{{TABLE_NAME}}_status ON {{TABLE_NAME}} (status);
CREATE INDEX IF NOT EXISTS idx_{{TABLE_NAME}}_deleted_at ON {{TABLE_NAME}} (deleted_at) WHERE deleted_at IS NOT NULL;

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_{{TABLE_NAME}}_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_{{TABLE_NAME}}_updated_at
    BEFORE UPDATE ON {{TABLE_NAME}}
    FOR EACH ROW
    EXECUTE FUNCTION update_{{TABLE_NAME}}_timestamp();
