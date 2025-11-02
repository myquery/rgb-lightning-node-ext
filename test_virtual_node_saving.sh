#!/bin/bash

echo "Testing virtual node ID saving in database..."

# Test 1: Call btc_balance with user_id to trigger virtual node ID saving
echo "1. Testing btc_balance endpoint with user_id..."
curl -X POST http://localhost:3001/btcbalance \
  -H "Content-Type: application/json" \
  -H "x-user-id: test_user_1" \
  -d '{"skip_sync": true}' | jq .

echo ""

# Test 2: Call asset_balance with user_id to trigger virtual node ID saving  
echo "2. Testing asset_balance endpoint with user_id..."
curl -X POST http://localhost:3001/assetbalance \
  -H "Content-Type: application/json" \
  -H "x-user-id: test_user_2" \
  -d '{"asset_id": "rgb1test"}' | jq .

echo ""

# Test 3: Call virtual_transfer to see virtual node IDs in transaction record
echo "3. Testing virtual_transfer endpoint..."
curl -X POST http://localhost:3001/virtual_transfer \
  -H "Content-Type: application/json" \
  -d '{
    "from_user_id": 1,
    "to_user_id": 2,
    "amount_sats": 1000
  }' | jq .

echo ""
echo "Virtual node ID saving test completed"