#!/bin/bash
# Chetna Launch Script

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# Get the directory of the script
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$DIR"

# Check if binary exists
if [ ! -f "./target/release/chetna" ]; then
    echo "Chetna binary not found. Please run ./install.sh first."
    exit 1
fi

# Set environment variables if needed (though Chetna loads them from ChetnaData/.env)
# export RUST_LOG=info

echo -e "${BLUE}🛑 Stopping any running Chetna instances...${NC}"
# Use pkill to find and kill processes running the chetna binary
pkill -f "./target/release/chetna" || true
sleep 1

echo -e "${BLUE}🧠 Starting Chetna Memory System...${NC}"

# Start Chetna
exec ./target/release/chetna
