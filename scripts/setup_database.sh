#!/bin/bash

# Master script to set up the database with routing rules and topic validations
# Usage: ./setup_database.sh [base_url]
# Example: ./setup_database.sh http://localhost:8080/api/v1

# Default base URL if not provided
BASE_URL=${1:-"http://localhost:8080/api/v1"}

echo "Setting up database with routing rules and topic validations"
echo "Base URL: $BASE_URL"
echo ""

# Make scripts executable
chmod +x "$(dirname "$0")/add_routing_rules.sh"
chmod +x "$(dirname "$0")/add_topic_validations.sh"
chmod +x "$(dirname "$0")/send_test_events.sh"

# Add routing rules
echo "Step 1: Adding routing rules"
"$(dirname "$0")/add_routing_rules.sh" "$BASE_URL"
echo ""

# Add topic validations
echo "Step 2: Adding topic validations"
"$(dirname "$0")/add_topic_validations.sh" "$BASE_URL"
echo ""

# Ask if user wants to send test events
read -p "Do you want to send test events? (y/n): " SEND_EVENTS
if [[ "$SEND_EVENTS" == "y" || "$SEND_EVENTS" == "Y" ]]; then
  echo "Step 3: Sending test events"
  "$(dirname "$0")/send_test_events.sh" "$BASE_URL"
  echo ""
fi

echo "Database setup completed successfully" 