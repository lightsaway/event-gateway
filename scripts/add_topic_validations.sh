#!/bin/bash

# Script to add topic validations to the event gateway
# Usage: ./add_topic_validations.sh [base_url]
# Example: ./add_topic_validations.sh http://localhost:8080/api/v1

# Default base URL if not provided
BASE_URL=${1:-"http://localhost:8080/api/v1"}

echo "Adding topic validations to $BASE_URL"

# Function to add a topic validation
add_validation() {
  local topic=$1
  local event_type=$2
  local event_version=$3
  local schema=$4

  echo "Adding validation for topic: $topic, event: $event_type, version: $event_version"
  
  response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/topic-validations" \
    -H "Content-Type: application/json" \
    -d "{
      \"topic\": \"$topic\",
      \"eventType\": \"$event_type\",
      \"eventVersion\": \"$event_version\",
      \"schema\": {
        \"name\": \"${event_type}_schema\",
        \"description\": \"Schema for ${event_type} events\",
        \"schema\": {
          \"type\": \"json\",
          \"data\": $schema
        },
        \"event_type\": \"$event_type\",
        \"event_version\": \"$event_version\",
        \"metadata\": {}
      }
    }")
  
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

# Add user validation schema
add_validation "users" "user.created" "1.0" '{
  "type": "object",
  "properties": {
    "id": {
      "type": "string"
    },
    "name": {
      "type": "string"
    },
    "email": {
      "type": "string",
      "format": "email"
    },
    "created_at": {
      "type": "string",
      "format": "date-time"
    }
  },
  "required": ["id", "name", "email", "created_at"]
}'

# Add order validation schema
add_validation "orders" "order.created" "1.0" '{
  "type": "object",
  "properties": {
    "id": {
      "type": "string"
    },
    "user_id": {
      "type": "string"
    },
    "items": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "product_id": {
            "type": "string"
          },
          "quantity": {
            "type": "integer",
            "minimum": 1
          },
          "price": {
            "type": "number",
            "minimum": 0
          }
        },
        "required": ["product_id", "quantity", "price"]
      }
    },
    "total_amount": {
      "type": "number",
      "minimum": 0
    },
    "created_at": {
      "type": "string",
      "format": "date-time"
    }
  },
  "required": ["id", "user_id", "items", "total_amount", "created_at"]
}'

# Add analytics validation schema
add_validation "analytics" "product.viewed" "1.0" '{
  "type": "object",
  "properties": {
    "product_id": {
      "type": "string"
    },
    "user_id": {
      "type": "string"
    },
    "session_id": {
      "type": "string"
    },
    "viewed_at": {
      "type": "string",
      "format": "date-time"
    },
    "source": {
      "type": "string",
      "enum": ["search", "category", "recommendation", "direct"]
    }
  },
  "required": ["product_id", "user_id", "viewed_at"]
}'

echo "Topic validations added successfully" 