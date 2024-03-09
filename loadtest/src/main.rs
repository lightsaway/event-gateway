use goose::prelude::*;
use serde_json::json;
use std::time::Duration;

async fn post_event(user: &mut GooseUser) -> TransactionResult {
    let request_body = &serde_json::json!({
        "id": "123e4567-e89b-12d3-a456-426614174000",
        "eventType": "test",
        "eventVersion": "1.0",
        "metadata": {
            "key1": "value1",
            "key2": "value2"
        },
        "dataType": "string",
        "data": {
            "type": "json",
            "content": {
                "field1": "example",
                "field2": 42
            }
        },
        "timestamp": "2023-01-28T12:00:00Z",
        "origin": "localhost"
    });

    let _response = user.post_json("/event", request_body).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    GooseAttack::initialize()?
        .register_scenario(
            scenario!("PostEvent")
                // After each transactions runs, sleep randomly from 5 to 15 seconds.
                .set_wait_time(Duration::from_secs(5), Duration::from_secs(15))?
                .register_transaction(
                    transaction!(post_event)
                        .set_name("Post event") // Optional: Set a name for the task
                        .set_weight(1)?, // Optional: Set a weight for the task
                ),
        )
        .execute()
        .await?;

    Ok(())
}
