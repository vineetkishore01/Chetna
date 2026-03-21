#!/bin/bash
# Chetna Interactive Setup
# Single script for all setup methods - lean and simple
# Usage: ./install.sh [--auto]
#
# For AI Agents: ./install.sh --auto
#   - Uses Docker if available, otherwise local
#   - Auto-installs Rust if needed
#   - Uses default embedding model
#   - No user interaction required

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
CHETNA_PORT="${CHETNA_PORT:-1987}"
OLLAMA_URL="${OLLAMA_URL:-http://localhost:11434}"
EMBEDDING_MODEL="${EMBEDDING_MODEL:-qwen3-embedding:4b}"
AUTO_MODE=false

# Parse arguments
if [[ "$1" == "--auto" ]]; then
    AUTO_MODE=true
    echo "🤖 Auto mode enabled - no user interaction required"
fi

# Helper functions
print_header() {
    echo -e "${BLUE}"
    echo "╔═══════════════════════════════════════════════════════════╗"
    echo "║              CHETNA - AI Memory System                   ║"
    echo "║                   Setup Wizard                           ║"
    echo "╚═══════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_step() { echo -e "${BLUE}▶ $1${NC}"; }
print_success() { echo -e "${GREEN}✓ $1${NC}"; }
print_warning() { echo -e "${YELLOW}⚠ $1${NC}"; }
print_error() { echo -e "${RED}✗ $1${NC}"; }

command_exists() { command -v "$1" &> /dev/null; }
ollama_running() { curl -s "$OLLAMA_URL/api/tags" &> /dev/null; }

# Ask yes/no question
ask_yes_no() {
    local prompt="$1"
    local default="${2:-y}"

    # Auto mode: always use default
    if [ "$AUTO_MODE" = true ]; then
        [ "$default" = "y" ] && return 0 || return 1
    fi

    while true; do
        if [ "$default" = "y" ]; then
            read -p "$prompt [Y/n] " -n 1 -r
        else
            read -p "$prompt [y/N] " -n 1 -r
        fi
        echo
        case $REPLY in
            [Yy]*) return 0 ;;
            [Nn]*) return 1 ;;
            *) [ -z "$REPLY" ] && [ "$default" = "y" ] && return 0 || return 1 ;;
        esac
    done
}

# Ask for text input with default
ask_input() {
    local prompt="$1"
    local default="$2"

    # Auto mode: always use default
    if [ "$AUTO_MODE" = true ]; then
        echo "$default"
        return
    fi

    read -p "$prompt [$default]: " input
    echo "${input:-$default}"
}

# Install Rust
install_rust() {
    print_step "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    [ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
    command_exists cargo && print_success "Rust installed" || return 1
}

# Install Ollama
install_ollama() {
    print_step "Installing Ollama..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        brew install ollama 2>/dev/null || curl -fsSL https://ollama.ai/install.sh | sh
    else
        curl -fsSL https://ollama.ai/install.sh | sh
    fi
    print_success "Ollama installed"
}

# Pull embedding model
pull_model() {
    local model="$1"
    print_step "Pulling embedding model: $model..."
    ollama pull "$model"
    print_success "Model pulled"
}

# Setup with Docker
setup_docker() {
    print_step "Setting up with Docker..."
    
    mkdir -p ChetnaData
    cat > ChetnaData/.env << EOF
CHETNA_PORT=$CHETNA_PORT
CHETNA_DB_PATH=./ChetnaData/chetna.db
EMBEDDING_PROVIDER=ollama
EMBEDDING_MODEL=$EMBEDDING_MODEL
EMBEDDING_BASE_URL=http://host.docker.internal:11434
CONSOLIDATION_INTERVAL=6
AUTO_DECAY_ENABLED=true
AUTO_FLUSH_ENABLED=true
MIN_IMPORTANCE_THRESHOLD=0.1
EOF
    
    docker-compose up -d
    
    print_step "Waiting for Chetna to start..."
    for i in {1..30}; do
        curl -s "http://localhost:$CHETNA_PORT/health" &> /dev/null && break
        sleep 1
    done
    
    curl -s "http://localhost:$CHETNA_PORT/health" &> /dev/null && \
        print_success "Chetna is running!" || \
        { print_error "Failed to start"; exit 1; }
}

# Setup locally
setup_local() {
    local ollama_url="$1"
    
    print_step "Building Chetna..."
    cargo build --release
    print_success "Build complete"
    
    mkdir -p ChetnaData/logs
    cat > ChetnaData/.env << EOF
CHETNA_PORT=$CHETNA_PORT
CHETNA_DB_PATH=./ChetnaData/chetna.db
EMBEDDING_PROVIDER=ollama
EMBEDDING_MODEL=$EMBEDDING_MODEL
EMBEDDING_BASE_URL=$ollama_url
CONSOLIDATION_INTERVAL=6
AUTO_DECAY_ENABLED=true
AUTO_FLUSH_ENABLED=true
MIN_IMPORTANCE_THRESHOLD=0.1
EOF
    
    print_step "Starting Chetna in background..."
    nohup ./target/release/chetna > ChetnaData/logs/chetna.log 2>&1 &
    sleep 3
    
    print_step "Waiting for Chetna to start..."
    for i in {1..30}; do
        curl -s "http://localhost:$CHETNA_PORT/health" &> /dev/null && break
        sleep 1
    done
    
    curl -s "http://localhost:$CHETNA_PORT/health" &> /dev/null && \
        print_success "Chetna is running!" || \
        { print_error "Failed to start. Check logs in ChetnaData/logs/chetna.log"; exit 1; }
}

# Main setup flow
print_header

echo ""

# Auto mode: Skip welcome, go straight to setup
if [ "$AUTO_MODE" = true ]; then
    echo "🤖 AI Agent Auto-Setup Mode"
    echo ""
else
    echo "Welcome to Chetna! Let's get you set up."
    echo ""
fi

# Check if already running
if curl -s "http://localhost:$CHETNA_PORT/health" &> /dev/null; then
    print_success "Chetna is already running at http://localhost:$CHETNA_PORT"
    exit 0
fi

# Auto mode: Check Docker and use if available
if [ "$AUTO_MODE" = true ]; then
    if command_exists docker && (command_exists docker-compose || docker compose version &> /dev/null); then
        print_step "Docker detected - using Docker setup"
        setup_docker
        exit 0
    else
        print_warning "Docker not available - using local setup"
    fi
fi

# Question 1: Docker or Local?
if [ "$AUTO_MODE" = false ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "SETUP METHOD"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Chetna can run in two ways:"
    echo ""
    echo "  1) Docker     - Isolated container, easy cleanup"
    echo "  2) Local      - Native performance, no Docker needed"
    echo ""

    if command_exists docker && (command_exists docker-compose || docker compose version &> /dev/null); then
        if ask_yes_no "Use Docker? (recommended)" "y"; then
            setup_docker
            exit 0
        fi
    else
        print_warning "Docker not available, using local installation"
    fi
fi

# Question 2: Rust installed?
if [ "$AUTO_MODE" = false ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "RUST INSTALLATION"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
fi

if command_exists cargo; then
    if [ "$AUTO_MODE" = false ]; then
        print_success "Rust found: $(rustc --version)"
    fi
else
    print_error "Rust is not installed"

    if [ "$AUTO_MODE" = true ]; then
        # Auto mode: install without asking
        install_rust || { print_error "Installation failed"; exit 1; }
    else
        # Interactive mode: ask user
        echo ""
        echo "Rust is required to build Chetna."
        echo ""
        if ask_yes_no "Install Rust now?" "y"; then
            install_rust || { print_error "Installation failed"; exit 1; }
        else
            print_error "Cannot proceed without Rust"
            exit 1
        fi
    fi
fi

# Question 3: Ollama setup
if [ "$AUTO_MODE" = false ]; then
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "OLLAMA CONFIGURATION"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Chetna uses Ollama for AI embeddings (semantic search)."
    echo "Without Ollama, only basic keyword search will work."
    echo ""
fi

OLLAMA_CHOICE=""

# Check local Ollama
if command_exists ollama && ollama_running; then
    if [ "$AUTO_MODE" = false ]; then
        print_success "Local Ollama found and running"
        if ask_yes_no "Use local Ollama?" "y"; then
            OLLAMA_CHOICE="local"
        fi
    else
        OLLAMA_CHOICE="local"  # Auto mode: use if available
    fi
elif command_exists ollama; then
    if [ "$AUTO_MODE" = false ]; then
        print_warning "Local Ollama found but not running"
        if ask_yes_no "Start Ollama now?" "y"; then
            ollama serve &
            sleep 3
            OLLAMA_CHOICE="local"
        fi
    else
        ollama serve &
        sleep 3
        OLLAMA_CHOICE="local"  # Auto mode: start it
    fi
fi

# If not using local, check for remote or install
if [ -z "$OLLAMA_CHOICE" ]; then
    if [ "$AUTO_MODE" = true ]; then
        # Auto mode: try remote, then install, then skip
        if [ -n "$OLLAMA_URL" ] && curl -s "$OLLAMA_URL/api/tags" &> /dev/null; then
            print_step "Using remote Ollama at $OLLAMA_URL"
            OLLAMA_CHOICE="remote"
        else
            print_step "Installing Ollama..."
            install_ollama
            ollama serve &
            sleep 3
            OLLAMA_CHOICE="local"
        fi
    else
        # Interactive mode: ask user
        if ask_yes_no "Use remote Ollama server?" "n"; then
            OLLAMA_URL=$(ask_input "Enter Ollama server URL" "http://localhost:11434")
            OLLAMA_CHOICE="remote"
        fi
    fi
fi

# If still no choice in interactive mode
if [ -z "$OLLAMA_CHOICE" ] && [ "$AUTO_MODE" = false ]; then
    echo ""
    echo "Options:"
    echo "  1) Install Ollama locally (recommended)"
    echo "  2) Skip Ollama (keyword search only)"
    echo ""

    if ask_yes_no "Install Ollama now?" "y"; then
        install_ollama
        ollama serve &
        sleep 3
        OLLAMA_CHOICE="local"
    else
        print_warning "Skipping Ollama - semantic search disabled"
        OLLAMA_CHOICE="none"
    fi
fi

# Auto mode: default to local if still unset
if [ -z "$OLLAMA_CHOICE" ] && [ "$AUTO_MODE" = true ]; then
    print_warning "No Ollama available - installing locally"
    install_ollama
    ollama serve &
    sleep 3
    OLLAMA_CHOICE="local"
fi

# Question 4: Embedding model (if using Ollama)
if [ "$OLLAMA_CHOICE" != "none" ]; then
    if [ "$AUTO_MODE" = false ]; then
        echo ""
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "EMBEDDING MODEL"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
        echo "Recommended models:"
        echo "  1) qwen3-embedding:4b  - Best quality (4GB RAM)"
        echo "  2) nomic-embed-text    - Good balance (500MB RAM)"
        echo "  3) gemma3-embed-e2b    - Lightweight (256MB RAM)"
        echo ""

        echo "Select model (1-3) or enter custom name:"
        read -r model_choice
        case $model_choice in
            1) EMBEDDING_MODEL="qwen3-embedding:4b" ;;
            2) EMBEDDING_MODEL="nomic-embed-text" ;;
            3) EMBEDDING_MODEL="gemma3-embed-e2b" ;;
            *) EMBEDDING_MODEL="${model_choice:-qwen3-embedding:4b}" ;;
        esac
    fi

    # Check if model exists
    if ollama list 2>/dev/null | grep -q "$EMBEDDING_MODEL"; then
        print_success "Model already installed"
    else
        if [ "$AUTO_MODE" = false ]; then
            if ask_yes_no "Pull model now? ($(numfmt --to=iec-i --suffix=B 4000000000 2>/dev/null || echo '4GB'))" "y"; then
                pull_model "$EMBEDDING_MODEL"
            fi
        else
            # Auto mode: always pull
            pull_model "$EMBEDDING_MODEL"
        fi
    fi
fi

# Final setup
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "FINAL SETUP"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [ "$OLLAMA_CHOICE" = "none" ]; then
    print_warning "Running without Ollama - semantic search disabled"
    setup_local "http://localhost:11434"
else
    setup_local "$OLLAMA_URL"
fi

# Success message
echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                    SETUP COMPLETE                         ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Chetna is running at:"
echo "  🌐 Dashboard: http://localhost:$CHETNA_PORT"
echo "  📡 API:       http://localhost:$CHETNA_PORT/api"
echo "  🔗 MCP:       http://localhost:$CHETNA_PORT/mcp"
echo ""
echo "Configuration:"
echo "  Database:   ./ChetnaData/chetna.db"
echo "  Ollama:     $OLLAMA_URL"
echo "  Embedding:  $EMBEDDING_MODEL"
echo ""
echo "To stop: kill \$(lsof -ti:$CHETNA_PORT)"
echo ""
echo "Run manually: ./chetna.sh"
echo "View logs:    tail -f ChetnaData/logs/chetna.log"
echo ""
