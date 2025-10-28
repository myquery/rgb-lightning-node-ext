#!/bin/bash

# RGB Lightning Node Startup Script with Auto-Unblock

echo "🚀 Starting RGB Lightning Node with Telegram Integration..."

# Function to check if service is running
check_service() {
    local url=$1
    local name=$2
    if curl -s "$url" > /dev/null 2>&1; then
        echo "✅ $name is running"
        return 0
    else
        echo "❌ $name is not running"
        return 1
    fi
}

# Check and kill existing processes on ports 3001 and 9735
echo "🔍 Checking for existing processes..."
EXISTING_3001=$(lsof -ti:3001)
EXISTING_9735=$(lsof -ti:9735)

if [ ! -z "$EXISTING_3001" ]; then
    echo "🛑 Killing existing process on port 3001 (PID: $EXISTING_3001)"
    kill -9 $EXISTING_3001
    sleep 2
fi

if [ ! -z "$EXISTING_9735" ]; then
    echo "🛑 Killing existing process on port 9735 (PID: $EXISTING_9735)"
    kill -9 $EXISTING_9735
    sleep 2
fi

# Check if node is already running and unlocked
if check_service "http://localhost:3001/nodeinfo" "RGB Lightning Node (unlocked)"; then
    echo "✅ RGB Lightning Node is already running and unlocked!"
    echo "🔗 API: http://localhost:3001"
    echo "📱 Telegram Integration: http://localhost:3001/telegram"
    exit 0
fi

# Start RGB Lightning Node
echo "📡 Starting RGB Lightning Node..."
cd /home/oem/apps/rgb-lightning-node

# Load environment variables
export $(cat .env | xargs)

# Start RGB Lightning Node daemon
cargo run -- dataldk0/ \
    --daemon-listening-port 3001 \
    --ldk-peer-listening-port 9735 \
    --network testnet &

LIGHTNING_PID=$!
echo "✅ RGB Lightning Node started with PID: $LIGHTNING_PID"

# Wait for Lightning Node to be ready
echo "⏳ Waiting for RGB Lightning Node to be ready..."
until check_service "http://localhost:3001/networkinfo" "RGB Lightning Node"; do
    sleep 2
done

# Check if already unlocked
if check_service "http://localhost:3001/nodeinfo" "RGB Lightning Node (checking unlock status)"; then
    echo "✅ Node is already unlocked!"
else
    # Unlock the RGB Lightning Node
    echo "🔓 Unlocking RGB Lightning Node..."
    curl -X POST http://localhost:3001/unlock \
      -H "Content-Type: application/json" \
      -d '{
        "password": "testtest",
        "bitcoind_rpc_username": "user",
        "bitcoind_rpc_password": "password",
        "bitcoind_rpc_host": "electrum.iriswallet.com",
        "bitcoind_rpc_port": 18332,
        "indexer_url": "ssl://electrum.iriswallet.com:50013",
        "proxy_endpoint": "rpcs://proxy.iriswallet.com/0.2/json-rpc",
        "announce_addresses": []
      }'
fi

echo ""
echo "✅ RGB Lightning Node is ready!"
echo ""
echo "📋 Node Status:"
echo "  🔗 RGB Lightning Node: http://localhost:3001"
echo "  📱 Telegram Integration: http://localhost:3001/telegram"
echo "  🗄️  Database: PostgreSQL Multi-User"
echo ""
echo "📋 Available Features:"
echo "  • Real RGB token operations"
echo "  • Lightning Network integration"
echo "  • Multi-user wallet isolation"
echo "  • Telegram bot API endpoints"
echo ""
echo "🛑 To stop:"
echo "  kill $LIGHTNING_PID"
echo ""
echo "💡 RGB Lightning Node ready for Telegram bot connections!"

# Keep script running
wait