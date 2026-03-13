#!/bin/bash
# Chetna Launcher Script
# Usage: ./launch.sh [options]
# Options:
#   --no-kill    Don't kill existing process
#   --api        Just check if running, don't start
#   --stop       Just stop the running instance

set -e

PORT=${CHETNA_PORT:-1987}
DB_PATH=${CHETNA_DB_PATH:-"./ChetnaData/chetna.db"}
HOST=${CHETNA_HOST:-"0.0.0.0"}

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Find and kill existing process
kill_existing() {
    if [ "$1" = "--no-kill" ]; then
        return 0
    fi
    
    # Find process by port
    PID=$(lsof -ti:$PORT 2>/dev/null || true)
    
    if [ -n "$PID" ]; then
        echo -e "${YELLOW}Found existing Chetna on port $PORT (PID: $PID)${NC}"
        echo "Killing process..."
        kill $PID 2>/dev/null || true
        sleep 2
        
        # Force kill if still running
        if kill -0 $PID 2>/dev/null; then
            kill -9 $PID 2>/dev/null || true
            sleep 1
        fi
        echo -e "${GREEN}Process killed${NC}"
    fi
}

# Check if running
check_running() {
    PID=$(lsof -ti:$PORT 2>/dev/null || true)
    if [ -n "$PID" ]; then
        echo "Chetna is running on port $PORT (PID: $PID)"
        return 0
    else
        echo "Chetna is not running"
        return 1
    fi
}

# Stop Chetna
stop_chetna() {
    PID=$(lsof -ti:$PORT 2>/dev/null || true)
    if [ -n "$PID" ]; then
        echo "Stopping Chetna (PID: $PID)..."
        kill $PID 2>/dev/null || true
        sleep 2
        echo -e "${GREEN}Chetna stopped${NC}"
    else
        echo "Chetna is not running"
    fi
}

# Start Chetna
start_chetna() {
    echo "Starting Chetna..."
    echo "  Host: $HOST"
    echo "  Port: $PORT"
    echo "  Database: $DB_PATH"
    
    # Ensure data directory exists
    mkdir -p "$(dirname "$DB_PATH")"
    
    # Start the application
    cargo run &
    PID=$!
    
    echo "Chetna started (PID: $PID)"
    echo ""
    echo "Dashboard: http://localhost:$PORT"
    echo "API:       http://localhost:$PORT/api"
    echo ""
    echo "Press Ctrl+C to stop"
    
    # Wait for the process
    wait $PID
}

# API mode - for AI agents
api_start() {
    if check_running; then
        echo '{"status":"running","port":'$PORT',"message":"Chetna already running"}'
        exit 0
    fi
    
    kill_existing
    start_chetna &
    
    # Wait for server to start
    echo "Waiting for server to start..."
    for i in {1..30}; do
        if curl -s http://localhost:$PORT/health >/dev/null 2>&1; then
            echo '{"status":"started","port":'$PORT',"message":"Chetna started successfully"}'
            exit 0
        fi
        sleep 1
    done
    
    echo '{"status":"error","message":"Timeout waiting for Chetna to start"}'
    exit 1
}

# Main
case "${1:-}" in
    --api)
        api_start
        ;;
    --stop)
        stop_chetna
        ;;
    --check|-c)
        check_running
        ;;
    --help|-h)
        echo "Chetna Launcher"
        echo ""
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  (none)      Start Chetna (kills existing if running)"
        echo "  --no-kill   Start without killing existing process"
        echo "  --api       Start and wait for server (for AI agents)"
        echo "  --stop      Stop running Chetna"
        echo "  --check     Check if Chetna is running"
        echo "  --help      Show this help"
        ;;
    *)
        kill_existing "$@"
        start_chetna
        ;;
esac
