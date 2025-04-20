#!/bin/bash

# Script to send test events to the event gateway
# Usage: ./send_test_events.sh [base_url]
# Example: ./send_test_events.sh http://localhost:8080/api/v1

# Default base URL if not provided
BASE_URL=${1:-"http://localhost:8080/api/v1"}

echo "Sending test events to $BASE_URL"

# Function to send a test event
send_event() {
  local event_type=$1
  local event_version=$2
  local payload=$3

  echo "Sending event: $event_type (v$event_version)"
  
  response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/event" \
    -H "Content-Type: application/json" \
    -d "$payload")
  
  # Extract status code from the last line
  status_code=$(echo "$response" | tail -n1)
  # Extract response body (everything except the last line)
  body=$(echo "$response" | sed '$d')
  
  if [ "$status_code" -eq 200 ] || [ "$status_code" -eq 201 ] || [ "$status_code" -eq 204 ]; then
    echo "  Success (HTTP $status_code)"
  else
    echo "  Error (HTTP $status_code): $body"
  fi
  
  echo ""
}

# Send user created event
send_event "user.created" "1.0" '{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "eventType": "user.created",
  "eventVersion": "1.0",
  "metadata": {
    "source": "test-script",
    "environment": "test"
  },
  "data": {
    "type": "json",
    "content": {
      "id": "user-123",
      "name": "John Doe",
      "email": "john@example.com",
      "created_at": "2023-01-01T12:00:00Z"
    }
  }
}'

# Send user updated event
send_event "user.updated" "1.0" '{
  "id": "123e4567-e89b-12d3-a456-426614174001",
  "eventType": "user.updated",
  "eventVersion": "1.0",
  "metadata": {
    "source": "test-script",
    "environment": "test"
  },
  "data": {
    "type": "json",
    "content": {
      "id": "user-123",
      "name": "John Smith",
      "email": "john.smith@example.com",
      "updated_at": "2023-01-02T12:00:00Z"
    }
  }
}'

# Send order created event
send_event "order.created" "1.0" '{
  "id": "123e4567-e89b-12d3-a456-426614174002",
  "eventType": "order.created",
  "eventVersion": "1.0",
  "metadata": {
    "source": "test-script",
    "environment": "test"
  },
  "data": {
    "type": "json",
    "content": {
      "id": "order-456",
      "user_id": "user-123",
      "items": [
        {
          "product_id": "prod-789",
          "quantity": 2,
          "price": 19.99
        }
      ],
      "total_amount": 39.98,
      "created_at": "2023-01-03T12:00:00Z"
    }
  }
}'

# Send order completed event
send_event "order.completed" "1.0" '{
  "id": "123e4567-e89b-12d3-a456-426614174003",
  "eventType": "order.completed",
  "eventVersion": "1.0",
  "metadata": {
    "source": "test-script",
    "environment": "test"
  },
  "data": {
    "type": "json",
    "content": {
      "id": "order-456",
      "user_id": "user-123",
      "completed_at": "2023-01-04T12:00:00Z",
      "status": "completed"
    }
  }
}'

# Send product viewed event
send_event "product.viewed" "1.0" '{
  "id": "123e4567-e89b-12d3-a456-426614174004",
  "eventType": "product.viewed",
  "eventVersion": "1.0",
  "metadata": {
    "source": "test-script",
    "environment": "test"
  },
  "data": {
    "type": "json",
    "content": {
      "product_id": "prod-789",
      "user_id": "user-123",
      "session_id": "sess-abc",
      "viewed_at": "2023-01-05T12:00:00Z",
      "source": "search"
    }
  }
}'

echo "Test events sent successfully" 