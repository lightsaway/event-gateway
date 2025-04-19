CREATE TABLE IF NOT EXISTS routing_rules (
    id UUID PRIMARY KEY,
    order_num INTEGER NOT NULL,
    topic TEXT NOT NULL,
    description TEXT,
    event_version_condition JSONB NOT NULL,
    event_type_condition JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS topic_validations (
    id UUID PRIMARY KEY,
    topic TEXT NOT NULL,
    schema JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_routing_rules_order ON routing_rules(order_num);
CREATE INDEX IF NOT EXISTS idx_topic_validations_topic ON topic_validations(topic); 