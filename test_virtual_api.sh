#!/bin/bash

# Test script for virtual API endpoints with user_id synchronization
# This tests the bitMaskRGB <-> RGB Lightning Node service integration

echo "Testing Virtual API Endpoints with user_id parameter..."

# Test 1: Virtual RGB Invoice with user_id
echo "1. Testing virtual_rgbinvoice with user_id..."
curl -X POST -H "Content-Type: application/json" \
    -d '{"user_id": "123456789", "asset_id": "BTC", "duration_seconds": 3600}' \
    http://localhost:3001/virtual_rgbinvoice 2>/dev/null | jq . || echo "Node not running or endpoint failed"

echo ""

# Test 2: Virtual Asset Balance with user_id  
echo "2. Testing virtual_assetbalance with user_id..."
curl -X POST -H "Content-Type: application/json" \
    -d '{"user_id": "123456789", "asset_id": "BTC"}' \
    http://localhost:3001/virtual_assetbalance 2>/dev/null | jq . || echo "Node not running or endpoint failed"

echo ""

# Test 3: Virtual Send Payment with user_id
echo "3. Testing virtual_sendpayment with user_id..."
curl -X POST -H "Content-Type: application/json" \
    -d '{"user_id": "123456789", "invoice": "lnbc1000n1...", "amt_msat": 1000}' \
    http://localhost:3001/virtual_sendpayment 2>/dev/null | jq . || echo "Node not running or endpoint failed"

echo ""

# Test 4: Virtual Transfer between users
echo "4. Testing virtual_transfer between users..."
curl -X POST -H "Content-Type: application/json" \
    -d '{"from_user_id": 123456789, "to_user_id": 987654321, "amount_sats": 1000}' \
    http://localhost:3001/virtual_transfer 2>/dev/null | jq . || echo "Node not running or endpoint failed"

echo ""
echo "Virtual API test completed. If node is running and unlocked, you should see JSON responses above."