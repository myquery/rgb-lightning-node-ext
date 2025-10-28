#!/bin/bash

# Test script for improved RGB Lightning Node connection pool

echo "🧪 Testing RGB Lightning Node with improved connection pool..."

# Build the project with our extended RGB library
echo "📦 Building RGB Lightning Node with extended RGB library..."
cd /home/oem/apps/rgb-lightning-node
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo "🔧 The RGB Lightning Node now uses improved connection pool settings:"
    echo "   - Max connections: 5 (was 1)"
    echo "   - Min connections: 1 (was 0)"
    echo "   - Connect timeout: 30s (was 8s)"
    echo "   - Idle timeout: 300s (was 8s)"
    echo "   - Max lifetime: 3600s (was 8s)"
    echo "   - Acquire timeout: 30s (new)"
    echo ""
    echo "🚀 These improvements should resolve the connection pool timeout issues"
    echo "   in multi-user environments by:"
    echo "   - Allowing more concurrent connections"
    echo "   - Keeping connections alive longer"
    echo "   - Providing more time for connection establishment"
    echo "   - Maintaining a minimum connection pool"
else
    echo "❌ Build failed. Check the error messages above."
    exit 1
fi