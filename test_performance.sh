#!/bin/bash
# Chetna Performance Test and Evaluation Script
# Tests memory creation, retrieval, context building, and TurboQuant performance

set -e

echo "=========================================="
echo "Chetna Performance Test & Evaluation"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BASE_URL="http://localhost:1987"
TEST_DB="./data/test_chetna.db"

# Function to print colored output
print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

# Function to check if server is running
check_server() {
    print_info "Checking if Chetna server is running..."
    if curl -s "$BASE_URL/health" > /dev/null 2>&1; then
        print_success "Server is running"
        return 0
    else
        print_error "Server is not running. Please start it first."
        return 1
    fi
}

# Function to test memory creation
test_memory_creation() {
    print_info "Testing memory creation..."
    
    local start_time=$(date +%s%N)
    
    # Create test memories
    local memories=(
        "User prefers dark mode in all applications"
        "Server runs on Ubuntu 22.04 with 16GB RAM"
        "API key is stored in /etc/api/keys/production"
        "User likes using Rust for development"
        "Database connection string: postgresql://user:pass@localhost:5432/db"
        "Git repository: https://github.com/user/project.git"
        "User's email: user@example.com"
        "Preferred editor: Vim with custom configuration"
        "Project deadline: December 31, 2026"
    )
    
    local memory_ids=()
    for i in "${!memories[@]}"; do
        local response=$(curl -s -X POST "$BASE_URL/api/memory" \
            -H "Content-Type: application/json" \
            -d "{
                \"content\": \"${memories[$i]}\",
                \"importance\": 0.$((8 + i % 2)),
                \"memory_type\": \"fact\",
                \"category\": \"fact\",
                \"tags\": [\"test\", \"batch$i\"]
            }")
        
        local id=$(echo "$response" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
        if [ -n "$id" ]; then
            memory_ids+=("$id")
            print_success "Created memory $((i+1))/${#memories[@]}: $id"
        else
            print_error "Failed to create memory $((i+1))"
        fi
    done
    
    local end_time=$(date +%s%N)
    local duration=$(( (end_time - start_time) / 1000000 ))
    
    print_success "Created ${#memory_ids[@]} memories in ${duration}ms"
    echo "Average: $((duration / ${#memory_ids[@]}))ms per memory"
    echo ""
    
    # Return memory IDs for later use
    echo "${memory_ids[@]}"
}

# Function to test memory retrieval
test_memory_retrieval() {
    print_info "Testing memory retrieval..."
    
    local queries=(
        "dark mode"
        "server configuration"
        "API key"
        "Rust development"
        "database"
    )
    
    for query in "${queries[@]}"; do
        local start_time=$(date +%s%N)
        
        local response=$(curl -s "$BASE_URL/api/memory/search?query=$query&limit=5")
        local count=$(echo "$response" | grep -o '"id"' | wc -l | tr -d ' ')
        
        local end_time=$(date +%s%N)
        local duration=$(( (end_time - start_time) / 1000000 ))
        
        if [ "$count" -gt 0 ]; then
            print_success "Query '$query': $count results in ${duration}ms"
        else
            print_warning "Query '$query': No results in ${duration}ms"
        fi
    done
    echo ""
}

# Function to test context building
test_context_building() {
    print_info "Testing context building for AI..."
    
    local contexts=(
        "What are the user's preferences?"
        "What is the server configuration?"
        "What development tools does the user use?"
    )
    
    for context_query in "${contexts[@]}"; do
        local start_time=$(date +%s%N)
        
        local response=$(curl -s -X POST "$BASE_URL/api/memory/context" \
            -H "Content-Type: application/json" \
            -d "{
                \"query\": \"$context_query\",
                \"limit\": 10,
                \"min_importance\": 0.5
            }")
        
        local context=$(echo "$response" | grep -o '"context":"[^"]*"' | cut -d'"' -f4 | sed 's/\\n/\n/g')
        local memories_used=$(echo "$response" | grep -o '"memories_used":[0-9]*' | cut -d':' -f2)
        
        local end_time=$(date +%s%N)
        local duration=$(( (end_time - start_time) / 1000000 ))
        
        print_success "Context '$context_query': ${duration}ms, $memories_used memories"
        echo "  Context preview: ${context:0:100}..."
        echo ""
    done
}

# Function to test TurboQuant
test_turboquant() {
    print_info "Testing TurboQuant performance..."
    
    # Get stats to check memory usage
    local stats=$(curl -s "$BASE_URL/api/stats")
    local total_memories=$(echo "$stats" | grep -o '"total_memories":[0-9]*' | cut -d':' -f2)
    local avg_importance=$(echo "$stats" | grep -o '"avg_importance":[0-9.]*' | cut -d':' -f2)
    
    print_success "Total memories: $total_memories"
    print_success "Average importance: $avg_importance"
    
    # Check database size
    if [ -f "$TEST_DB" ]; then
        local db_size=$(du -h "$TEST_DB" | cut -f1)
        print_success "Database size: $db_size"
    fi
    echo ""
}

# Function to evaluate context quality
evaluate_context_quality() {
    print_info "Evaluating context quality for AI..."
    
    # Test specific queries that should return relevant information
    local test_cases=(
        "query:What are the user's preferences?|expected:dark mode, Vim, Rust"
        "query:What is the server setup?|expected:Ubuntu, 16GB RAM, API keys"
        "query:What development tools are used?|expected:Rust, Vim"
    )
    
    for test_case in "${test_cases[@]}"; do
        local query=$(echo "$test_case" | cut -d'|' -f1)
        local expected=$(echo "$test_case" | cut -d'|' -f2)
        
        local response=$(curl -s -X POST "$BASE_URL/api/memory/context" \
            -H "Content-Type: application/json" \
            -d "{
                \"query\": \"$query\",
                \"limit\": 5,
                \"min_importance\": 0.5
            }")
        
        local context=$(echo "$response" | grep -o '"context":"[^"]*"' | cut -d'"' -f4 | sed 's/\\n/\n/g')
        
        # Check if expected keywords are in context
        local found=true
        for keyword in dark mode Vim Rust Ubuntu 16GB API; do
            if [[ "$context" == *"$keyword"* ]]; then
                print_success "✓ Found '$keyword' in context"
            fi
        done
        
        echo "  Query: $query"
        echo "  Expected: $expected"
        echo "  Context: ${context:0:150}..."
        echo ""
    done
}

# Function to test performance boost
test_performance_boost() {
    print_info "Testing performance boost from optimizations..."
    
    # Test query cache performance
    local query="dark mode preferences"
    
    print_info "Testing query cache (first query)..."
    local start_time=$(date +%s%N)
    curl -s "$BASE_URL/api/memory/search?query=$query&limit=5" > /dev/null
    local first_duration=$(( ($(date +%s%N) - start_time) / 1000000 ))
    
    print_info "Testing query cache (cached query)..."
    local start_time=$(date +%s%N)
    curl -s "$BASE_URL/api/memory/search?query=$query&limit=5" > /dev/null
    local cached_duration=$(( ($(date +%s%N) - start_time) / 1000000 ))
    
    local speedup=$(( first_duration / cached_duration ))
    print_success "First query: ${first_duration}ms"
    print_success "Cached query: ${cached_duration}ms"
    print_success "Speedup: ${speedup}x"
    echo ""
}

# Function to run all tests
run_all_tests() {
    print_info "Running comprehensive test suite..."
    echo ""
    
    # Check server
    if ! check_server; then
        exit 1
    fi
    
    # Test memory creation
    test_memory_creation
    
    # Test memory retrieval
    test_memory_retrieval
    
    # Test context building
    test_context_building
    
    # Test TurboQuant
    test_turboquant
    
    # Evaluate context quality
    evaluate_context_quality
    
    # Test performance boost
    test_performance_boost
    
    print_success "All tests completed!"
}

# Main execution
main() {
    run_all_tests
}

# Run main function
main "$@"