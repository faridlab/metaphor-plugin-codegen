-- Migration: Create additional indexes and triggers
-- Description: Creates performance indexes and audit triggers for {{MODULE_NAME}}
-- Version: 004
-- Created: {{CURRENT_TIMESTAMP}}

-- Create trigger function for audit logging
CREATE OR REPLACE FUNCTION audit_{{MODULE_NAME}}_changes()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO {{MODULE_NAME}}_audit (
            entity_id,
            operation,
            new_data,
            changed_fields,
            user_id,
            user_email,
            ip_address,
            user_agent,
            request_id,
            metadata
        ) VALUES (
            NEW.id,
            'INSERT',
            to_jsonb(NEW),
            ARRAY['created'],
            COALESCE(NULLIF(current_setting('app.current_user_id', true), '')::UUID, NULL),
            COALESCE(NULLIF(current_setting('app.current_user_email', true), ''), NULL),
            COALESCE(NULLIF(current_setting('app.current_ip', true), '')::INET, NULL),
            COALESCE(NULLIF(current_setting('app.current_user_agent', true), ''), NULL),
            COALESCE(NULLIF(current_setting('app.current_request_id', true), ''), NULL),
            COALESCE(NULLIF(current_setting('app.audit_metadata', true), '')::jsonb, '{}')
        );
        RETURN NEW;

    ELSIF TG_OP = 'UPDATE' THEN
        -- Only log if there are actual changes
        IF NEW IS DISTINCT FROM OLD THEN
            INSERT INTO {{MODULE_NAME}}_audit (
                entity_id,
                operation,
                old_data,
                new_data,
                changed_fields,
                user_id,
                user_email,
                ip_address,
                user_agent,
                request_id,
                metadata
            ) VALUES (
                NEW.id,
                CASE WHEN OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL THEN 'SOFT_DELETE'
                     WHEN OLD.deleted_at IS NOT NULL AND NEW.deleted_at IS NULL THEN 'RESTORE'
                     ELSE 'UPDATE'
                END,
                to_jsonb(OLD),
                to_jsonb(NEW),
                ARRAY[
                    CASE WHEN OLD.name IS DISTINCT FROM NEW.name THEN 'name' END,
                    CASE WHEN OLD.description IS DISTINCT FROM NEW.description THEN 'description' END,
                    CASE WHEN OLD.status IS DISTINCT FROM NEW.status THEN 'status' END,
                    CASE WHEN OLD.metadata IS DISTINCT FROM NEW.metadata THEN 'metadata' END,
                    CASE WHEN OLD.deleted_at IS DISTINCT FROM NEW.deleted_at THEN 'deleted_at' END
                ] FILTER (WHERE element IS NOT NULL),
                COALESCE(NULLIF(current_setting('app.current_user_id', true), '')::UUID, NULL),
                COALESCE(NULLIF(current_setting('app.current_user_email', true), ''), NULL),
                COALESCE(NULLIF(current_setting('app.current_ip', true), '')::INET, NULL),
                COALESCE(NULLIF(current_setting('app.current_user_agent', true), ''), NULL),
                COALESCE(NULLIF(current_setting('app.current_request_id', true), ''), NULL),
                COALESCE(NULLIF(current_setting('app.audit_metadata', true), '')::jsonb, '{}')
            );
        END IF;
        RETURN NEW;

    ELSIF TG_OP = 'DELETE' THEN
        INSERT INTO {{MODULE_NAME}}_audit (
            entity_id,
            operation,
            old_data,
            changed_fields,
            user_id,
            user_email,
            ip_address,
            user_agent,
            request_id,
            metadata
        ) VALUES (
            OLD.id,
            'DELETE',
            to_jsonb(OLD),
            ARRAY['deleted'],
            COALESCE(NULLIF(current_setting('app.current_user_id', true), '')::UUID, NULL),
            COALESCE(NULLIF(current_setting('app.current_user_email', true), ''), NULL),
            COALESCE(NULLIF(current_setting('app.current_ip', true), '')::INET, NULL),
            COALESCE(NULLIF(current_setting('app.current_user_agent', true), ''), NULL),
            COALESCE(NULLIF(current_setting('app.current_request_id', true), ''), NULL),
            COALESCE(NULLIF(current_setting('app.audit_metadata', true), '')::jsonb, '{}')
        );
        RETURN OLD;

    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create triggers for audit logging
CREATE TRIGGER trigger_{{MODULE_NAME}}_audit_insert
    AFTER INSERT ON {{MODULE_NAME}}
    FOR EACH ROW
    EXECUTE FUNCTION audit_{{MODULE_NAME}}_changes();

CREATE TRIGGER trigger_{{MODULE_NAME}}_audit_update
    AFTER UPDATE ON {{MODULE_NAME}}
    FOR EACH ROW
    EXECUTE FUNCTION audit_{{MODULE_NAME}}_changes();

CREATE TRIGGER trigger_{{MODULE_NAME}}_audit_delete
    AFTER DELETE ON {{MODULE_NAME}}
    FOR EACH ROW
    EXECUTE FUNCTION audit_{{MODULE_NAME}}_changes();

-- Create additional performance indexes for common query patterns

-- Index for status + created_at pagination (most common list query)
CREATE INDEX CONCURRENTLY idx_{{MODULE_NAME}}_status_created_pagination
ON {{MODULE_NAME}}(status, created_at DESC)
WHERE deleted_at IS NULL;

-- Index for name search with active filter
CREATE INDEX CONCURRENTLY idx_{{MODULE_NAME}}_active_name_search
ON {{MODULE_NAME}}(name)
WHERE deleted_at IS NULL AND status = 'active';

-- Partial index for recent entities (last 30 days)
CREATE INDEX CONCURRENTLY idx_{{MODULE_NAME}}_recent
ON {{MODULE_NAME}}(created_at DESC)
WHERE created_at >= NOW() - INTERVAL '30 days' AND deleted_at IS NULL;

-- Index for metadata-specific queries (commonly accessed metadata fields)
CREATE INDEX CONCURRENTLY idx_{{MODULE_NAME}}_metadata_category
ON {{MODULE_NAME}} USING gin((metadata->'category'))
WHERE metadata ? 'category';

-- Create optimized index for complex JSON queries
CREATE INDEX CONCURRENTLY idx_{{MODULE_NAME}}_metadata_tags
ON {{MODULE_NAME}} USING gin((metadata->'tags'));

-- Create function for full-text search
CREATE OR REPLACE FUNCTION search_{{MODULE_NAME}}(search_term TEXT)
RETURNS TABLE(id UUID, name VARCHAR, description TEXT, rank REAL) AS $$
BEGIN
    RETURN QUERY
    SELECT
        b.id,
        b.name,
        b.description,
        ts_rank(
            to_tsvector('english', b.name || ' ' || COALESCE(b.description, '')),
            plainto_tsquery('english', search_term)
        ) as rank
    FROM {{MODULE_NAME}} b
    WHERE
        b.deleted_at IS NULL AND
        to_tsvector('english', b.name || ' ' || COALESCE(b.description, '')) @@ plainto_tsquery('english', search_term)
    ORDER BY rank DESC, b.created_at DESC;
END;
$$ LANGUAGE plpgsql;

-- Create function for getting {{MODULE_NAME}} statistics
CREATE OR REPLACE FUNCTION get_{{MODULE_NAME}}_statistics()
RETURNS TABLE(
    total_count BIGINT,
    active_count BIGINT,
    inactive_count BIGINT,
    pending_count BIGINT,
    archived_count BIGINT,
    deleted_count BIGINT,
    created_today BIGINT,
    created_this_week BIGINT,
    created_this_month BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        (SELECT COUNT(*) FROM {{MODULE_NAME}}) as total_count,
        (SELECT COUNT(*) FROM {{MODULE_NAME}} WHERE status = 'active' AND deleted_at IS NULL) as active_count,
        (SELECT COUNT(*) FROM {{MODULE_NAME}} WHERE status = 'inactive' AND deleted_at IS NULL) as inactive_count,
        (SELECT COUNT(*) FROM {{MODULE_NAME}} WHERE status = 'pending' AND deleted_at IS NULL) as pending_count,
        (SELECT COUNT(*) FROM {{MODULE_NAME}} WHERE status = 'archived' AND deleted_at IS NULL) as archived_count,
        (SELECT COUNT(*) FROM {{MODULE_NAME}} WHERE deleted_at IS NOT NULL) as deleted_count,
        (SELECT COUNT(*) FROM {{MODULE_NAME}} WHERE DATE(created_at) = CURRENT_DATE) as created_today,
        (SELECT COUNT(*) FROM {{MODULE_NAME}} WHERE created_at >= DATE_TRUNC('week', CURRENT_DATE)) as created_this_week,
        (SELECT COUNT(*) FROM {{MODULE_NAME}} WHERE created_at >= DATE_TRUNC('month', CURRENT_DATE)) as created_this_month;
END;
$$ LANGUAGE plpgsql;

-- Add comments for new functions
COMMENT ON FUNCTION search_{{MODULE_NAME}} IS 'Performs full-text search on {{MODULE_NAME}} entities with ranking';
COMMENT ON FUNCTION get_{{MODULE_NAME}}_statistics IS 'Returns comprehensive statistics about {{MODULE_NAME}} entities';