#!/bin/bash

# Quick Start RGB Lightning Node
echo "🔥 RGB Lightning Node Quick Start"

# Check if node is already running
if curl -s http://localhost:3001/networkinfo > /dev/null 2>&1; then
    echo "✅ Node already running"
    echo "🔓 Checking if unlocked..."
    
    if curl -s http://localhost:3001/nodeinfo > /dev/null 2>&1; then
        echo "✅ Node is unlocked and ready!"
        echo "🌐 API: http://localhost:3001"
        echo "📱 Telegram Bot: Ready"
        exit 0
    else
        echo "🔓 Unlocking existing node..."
        ./unlock.sh
        exit 0
    fi
fi

# Start fresh
echo "🚀 Starting new node instance..."
./startup.sh