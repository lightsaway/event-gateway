-- migrations/20240101000000_create_initial_schema.sql
CREATE TABLE IF NOT EXISTS routing_rules (
    id UUID PRIMARY KEY,
    order_num INTEGER NOT NULL,
    topic TEXT NOT NULL,
    description TEXT,
    event_version_condition JSONB,
    event_type_condition JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS topic_validations (
    id UUID PRIMARY KEY,
    topic TEXT NOT NULL,
    schema JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
