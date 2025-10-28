#!/bin/bash

# DigitalOcean deployment script for RGB Lightning Node

echo "🚀 Deploying RGB Lightning Node to DigitalOcean..."

# Update system
sudo apt update && sudo apt upgrade -y

# Install additional dependencies for Ubuntu 24.04
sudo apt install -y build-essential pkg-config

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/download/v2.20.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Clone the repository
git clone https://github.com/RGB-Tools/rgb-lightning-node --recurse-submodules --shallow-submodules
cd rgb-lightning-node

# Build and start the service
docker-compose up -d --build

echo "✅ RGB Lightning Node deployed!"
echo "📡 API available at: http://$(curl -s ifconfig.me):3001"
echo "🔗 Lightning P2P port: 9735"
echo ""
echo "Next steps:"
echo "1. Initialize: curl -X POST http://$(curl -s ifconfig.me):3001/init -H 'Content-Type: application/json' -d '{\"password\":\"your-password\"}'"
echo "2. Unlock: curl -X POST http://$(curl -s ifconfig.me):3001/unlock -H 'Content-Type: application/json' -d '{...}'"