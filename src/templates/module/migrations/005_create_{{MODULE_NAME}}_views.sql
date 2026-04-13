-- Migration: Create database views for {{MODULE_NAME}}
-- Description: Creates optimized views for common query patterns
-- Version: 005
-- Created: {{CURRENT_TIMESTAMP}}

-- Create view for active {{MODULE_NAME}} entities (most commonly accessed)
CREATE OR REPLACE VIEW {{MODULE_NAME}}_active AS
SELECT
    id,
    name,
    description,
    status,
    metadata,
    created_at,
    updated_at
FROM {{MODULE_NAME}}
WHERE deleted_at IS NULL AND status = 'active';

-- Create view for {{MODULE_NAME}} statistics (materialized view for performance)
CREATE MATERIALIZED VIEW {{MODULE_NAME}}_stats AS
SELECT
    'total' as metric,
    COUNT(*)::BIGINT as count
FROM {{MODULE_NAME}}
WHERE deleted_at IS NULL

UNION ALL

SELECT
    'active' as metric,
    COUNT(*)::BIGINT as count
FROM {{MODULE_NAME}}
WHERE status = 'active' AND deleted_at IS NULL

UNION ALL

SELECT
    'inactive' as metric,
    COUNT(*)::BIGINT as count
FROM {{MODULE_NAME}}
WHERE status = 'inactive' AND deleted_at IS NULL

UNION ALL

SELECT
    'pending' as metric,
    COUNT(*)::BIGINT as count
FROM {{MODULE_NAME}}
WHERE status = 'pending' AND deleted_at IS NULL

UNION ALL

SELECT
    'archived' as metric,
    COUNT(*)::BIGINT as count
FROM {{MODULE_NAME}}
WHERE status = 'archived' AND deleted_at IS NULL

UNION ALL

SELECT
    'deleted' as metric,
    COUNT(*)::BIGINT as count
FROM {{MODULE_NAME}}
WHERE deleted_at IS NOT NULL

UNION ALL

SELECT
    'created_today' as metric,
    COUNT(*)::BIGINT as count
FROM {{MODULE_NAME}}
WHERE DATE(created_at) = CURRENT_DATE

UNION ALL

SELECT
    'created_this_week' as metric,
    COUNT(*)::BIGINT as count
FROM {{MODULE_NAME}}
WHERE created_at >= DATE_TRUNC('week', CURRENT_DATE)

UNION ALL

SELECT
    'created_this_month' as metric,
    COUNT(*)::BIGINT as count
FROM {{MODULE_NAME}}
WHERE created_at >= DATE_TRUNC('month', CURRENT_DATE);

-- Create unique index on materialized view for refresh operations
CREATE UNIQUE INDEX idx_{{MODULE_NAME}}_stats_unique ON {{MODULE_NAME}}_stats(metric);

-- Create view for {{MODULE_NAME}} with recent activity
CREATE OR REPLACE VIEW {{MODULE_NAME}}_with_activity AS
SELECT
    b.id,
    b.name,
    b.description,
    b.status,
    b.metadata,
    b.created_at,
    b.updated_at,
    -- Latest event information
    (SELECT event_type
     FROM {{MODULE_NAME}}_events e
     WHERE e.aggregate_id = b.id
     ORDER BY e.created_at DESC
     LIMIT 1) as latest_event_type,
    (SELECT created_at
     FROM {{MODULE_NAME}}_events e
     WHERE e.aggregate_id = b.id
     ORDER BY e.created_at DESC
     LIMIT 1) as latest_event_at,
    -- Event count
    (SELECT COUNT(*)
     FROM {{MODULE_NAME}}_events e
     WHERE e.aggregate_id = b.id) as event_count,
    -- Latest activity information
    (SELECT operation
     FROM {{MODULE_NAME}}_audit a
     WHERE a.entity_id = b.id
     ORDER BY a.timestamp DESC
     LIMIT 1) as latest_activity,
    (SELECT timestamp
     FROM {{MODULE_NAME}}_audit a
     WHERE a.entity_id = b.id
     ORDER BY a.timestamp DESC
     LIMIT 1) as latest_activity_at
FROM {{MODULE_NAME}} b
WHERE b.deleted_at IS NULL;

-- Create view for {{MODULE_NAME}} audit summary
CREATE OR REPLACE VIEW {{MODULE_NAME}}_audit_summary AS
SELECT
    b.id,
    b.name,
    b.status,
    b.created_at as created_at,
    b.updated_at as last_updated_at,
    -- Total changes
    (SELECT COUNT(*) FROM {{MODULE_NAME}}_audit a WHERE a.entity_id = b.id) as total_changes,
    -- Changes in last 24 hours
    (SELECT COUNT(*) FROM {{MODULE_NAME}}_audit a
     WHERE a.entity_id = b.id AND a.timestamp >= NOW() - INTERVAL '24 hours') as changes_24h,
    -- Changes in last 7 days
    (SELECT COUNT(*) FROM {{MODULE_NAME}}_audit a
     WHERE a.entity_id = b.id AND a.timestamp >= NOW() - INTERVAL '7 days') as changes_7d,
    -- Last change type
    (SELECT operation FROM {{MODULE_NAME}}_audit a
     WHERE a.entity_id = b.id
     ORDER BY a.timestamp DESC
     LIMIT 1) as last_operation,
    -- Last user to modify
    (SELECT user_email FROM {{MODULE_NAME}}_audit a
     WHERE a.entity_id = b.id AND a.user_email IS NOT NULL
     ORDER BY a.timestamp DESC
     LIMIT 1) as last_modified_by
FROM {{MODULE_NAME}} b
WHERE b.deleted_at IS NULL;

-- Create view for {{MODULE_NAME}} event timeline
CREATE OR REPLACE VIEW {{MODULE_NAME}}_event_timeline AS
SELECT
    e.id,
    e.aggregate_id,
    b.name as entity_name,
    e.event_type,
    e.event_data,
    e.event_version,
    e.created_at,
    e.metadata
FROM {{MODULE_NAME}}_events e
JOIN {{MODULE_NAME}} b ON e.aggregate_id = b.id
ORDER BY e.created_at DESC;

-- Create view for {{MODULE_NAME}} audit timeline
CREATE OR REPLACE VIEW {{MODULE_NAME}}_audit_timeline AS
SELECT
    a.id,
    a.entity_id,
    b.name as entity_name,
    a.operation,
    a.old_data,
    a.new_data,
    a.changed_fields,
    a.user_id,
    a.user_email,
    a.timestamp,
    a.ip_address,
    a.user_agent,
    a.request_id
FROM {{MODULE_NAME}}_audit a
LEFT JOIN {{MODULE_NAME}} b ON a.entity_id = b.id
ORDER BY a.timestamp DESC;

-- Create view for {{MODULE_NAME}} by metadata fields
CREATE OR REPLACE VIEW {{MODULE_NAME}}_by_metadata AS
SELECT
    b.id,
    b.name,
    b.description,
    b.status,
    b.created_at,
    b.updated_at,
    -- Extract common metadata fields for easier querying
    b.metadata->>'category' as category,
    b.metadata->>'tags' as tags,
    b.metadata->>'priority' as priority,
    b.metadata->>'owner' as owner
FROM {{MODULE_NAME}} b
WHERE b.deleted_at IS NULL;

-- Create view for {{MODULE_NAME}} performance metrics
CREATE MATERIALIZED VIEW {{MODULE_NAME}}_performance_metrics AS
SELECT
    DATE(created_at) as date,
    COUNT(*) as created_count,
    COUNT(*) FILTER (WHERE status = 'active') as active_count,
    AVG(EXTRACT(EPOCH FROM (updated_at - created_at))) as avg_update_time_seconds,
    MIN(updated_at) as first_update,
    MAX(updated_at) as last_update
FROM {{MODULE_NAME}}
GROUP BY DATE(created_at)
ORDER BY date DESC;

-- Create index on performance metrics
CREATE INDEX idx_{{MODULE_NAME}}_performance_metrics_date ON {{MODULE_NAME}}_performance_metrics(date);

-- Add comments for views
COMMENT ON VIEW {{MODULE_NAME}}_active IS 'View containing only active {{MODULE_NAME}} entities';
COMMENT ON VIEW {{MODULE_NAME}}_stats IS 'Materialized view with {{MODULE_NAME}} statistics';
COMMENT ON VIEW {{MODULE_NAME}}_with_activity IS 'View of {{MODULE_NAME}} entities with recent activity information';
COMMENT ON VIEW {{MODULE_NAME}}_audit_summary IS 'View summarizing audit information for each {{MODULE_NAME}} entity';
COMMENT ON VIEW {{MODULE_NAME}}_event_timeline IS 'Chronological view of all {{MODULE_NAME}} domain events';
COMMENT ON VIEW {{MODULE_NAME}}_audit_timeline IS 'Chronological view of all {{MODULE_NAME}} audit entries';
COMMENT ON VIEW {{MODULE_NAME}}_by_metadata IS 'View of {{MODULE_NAME}} entities with extracted metadata fields';
COMMENT ON VIEW {{MODULE_NAME}}_performance_metrics IS 'Materialized view with daily performance metrics';

-- Create function to refresh materialized views
CREATE OR REPLACE FUNCTION refresh_{{MODULE_NAME}}_materialized_views()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY {{MODULE_NAME}}_stats;
    REFRESH MATERIALIZED VIEW CONCURRENTLY {{MODULE_NAME}}_performance_metrics;
END;
$$ LANGUAGE plpgsql;

-- Create scheduled refresh function (for use with cron or pg_cron)
CREATE OR REPLACE FUNCTION schedule_{{MODULE_NAME}}_refresh()
RETURNS void AS $$
BEGIN
    PERFORM refresh_{{MODULE_NAME}}_materialized_views();
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION refresh_{{MODULE_NAME}}_materialized_views IS 'Refreshes all materialized views for {{MODULE_NAME}}';
COMMENT ON FUNCTION schedule_{{MODULE_NAME}}_refresh IS 'Scheduled refresh function for {{MODULE_NAME}} materialized views';