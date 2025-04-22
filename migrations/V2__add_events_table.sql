CREATE TABLE IF NOT EXISTS events (
    id UUID PRIMARY KEY,
    event_id UUID NOT NULL,
    event_type TEXT NOT NULL,
    event_version TEXT,
    routing_id UUID,
    destination_topic TEXT,
    failure_reason TEXT,
    stored_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    event_data JSONB NOT NULL,
    metadata JSONB,
    transport_metadata JSONB
);

CREATE INDEX IF NOT EXISTS idx_events_event_id ON events(event_id);
CREATE INDEX IF NOT EXISTS idx_events_event_type ON events(event_type);
CREATE INDEX IF NOT EXISTS idx_events_routing_id ON events(routing_id);
CREATE INDEX IF NOT EXISTS idx_events_stored_at ON events(stored_at); 