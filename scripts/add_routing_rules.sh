#!/bin/bash

# Script to add routing rules to the event gateway
# Usage: ./add_routing_rules.sh [base_url]
# Example: ./add_routing_rules.sh http://localhost:8080/api/v1

# Default base URL if not provided
BASE_URL=${1:-"http://localhost:8080/api/v1"}

echo "Adding routing rules to $BASE_URL"

# Function to add a routing rule
add_rule() {
  local name=$1
  local topic=$2
  local event_type=$3
  local event_version=$4
  local order=$5
  local description=$6

  echo "Adding rule: $name"
  
  response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/routing-rules" \
    -H "Content-Type: application/json" \
    -d "{
      \"topic\": \"$topic\",
      \"eventTypeCondition\": {
        \"type\": \"equals\",
        \"value\": \"$event_type\"
      },
      \"eventVersionCondition\": {
        \"type\": \"equals\",
        \"value\": \"$event_version\"
      },
      \"order\": $order,
      \"description\": \"$description\"
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

# Add sample routing rules
add_rule "user-created" "users" "user.created" "1.0" 0 "Route user creation events to users topic"
add_rule "user-updated" "users" "user.updated" "1.0" 1 "Route user update events to users topic"
add_rule "order-created" "orders" "order.created" "1.0" 0 "Route order creation events to orders topic"
add_rule "order-completed" "orders" "order.completed" "1.0" 1 "Route order completion events to orders topic"
add_rule "product-viewed" "analytics" "product.viewed" "1.0" 0 "Route product view events to analytics topic"

echo "Routing rules added successfully" 