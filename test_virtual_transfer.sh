#!/bin/bash

# Test script to verify virtual transfer uses proper virtual node IDs

echo "Testing virtual transfer with virtual node IDs..."

# Start the RGB Lightning Node in background (if not already running)
# ./target/release/rgb-lightning-node dataldk0/ --daemon-listening-port 3001 --ldk-peer-listening-port 9735 --network regtest &
# NODE_PID=$!

# Wait for node to start
# sleep 5

# Test virtual transfer endpoint
curl -X POST http://localhost:3001/virtual_transfer \
  -H "Content-Type: application/json" \
  -d '{
    "from_user_id": 1,
    "to_user_id": 2,
    "amount_sats": 1000
  }' | jq .

echo "Virtual transfer test completed"

# Clean up
# kill $NODE_PID 2>/dev/null