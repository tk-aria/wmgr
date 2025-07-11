#!/bin/bash
# Development build script with various build modes

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
MODE=${1:-"debug"}
FEATURES=${2:-""}

case "${MODE}" in
    "debug"|"dev")
        echo -e "${BLUE}Building in debug mode...${NC}"
        cargo build ${FEATURES:+--features "$FEATURES"}
        ;;
    "release")
        echo -e "${BLUE}Building in release mode...${NC}"
        cargo build --release ${FEATURES:+--features "$FEATURES"}
        ;;
    "test")
        echo -e "${BLUE}Running tests...${NC}"
        cargo test ${FEATURES:+--features "$FEATURES"}
        ;;
    "check")
        echo -e "${BLUE}Running cargo check...${NC}"
        cargo check ${FEATURES:+--features "$FEATURES"}
        ;;
    "clippy")
        echo -e "${BLUE}Running clippy...${NC}"
        cargo clippy --all-targets --all-features -- -D warnings
        ;;
    "fmt")
        echo -e "${BLUE}Running rustfmt...${NC}"
        cargo fmt --all
        ;;
    "audit")
        echo -e "${BLUE}Running security audit...${NC}"
        cargo audit
        ;;
    "bench")
        echo -e "${BLUE}Running benchmarks...${NC}"
        cargo bench ${FEATURES:+--features "$FEATURES"}
        ;;
    "doc")
        echo -e "${BLUE}Building documentation...${NC}"
        cargo doc --open ${FEATURES:+--features "$FEATURES"}
        ;;
    "clean")
        echo -e "${YELLOW}Cleaning build artifacts...${NC}"
        cargo clean
        ;;
    "all")
        echo -e "${BLUE}Running full validation pipeline...${NC}"
        
        echo -e "${YELLOW}1. Formatting...${NC}"
        cargo fmt --all -- --check
        
        echo -e "${YELLOW}2. Clippy...${NC}"
        cargo clippy --all-targets --all-features -- -D warnings
        
        echo -e "${YELLOW}3. Tests...${NC}"
        cargo test --all-features
        
        echo -e "${YELLOW}4. Documentation...${NC}"
        cargo doc --all-features --no-deps
        
        echo -e "${YELLOW}5. Release build...${NC}"
        cargo build --release
        
        echo -e "${GREEN}✓ All checks passed!${NC}"
        ;;
    "install")
        echo -e "${BLUE}Installing tsrc locally...${NC}"
        cargo install --path . --force
        ;;
    *)
        echo -e "${RED}Unknown mode: ${MODE}${NC}"
        echo "Usage: $0 [mode] [features]"
        echo "Modes:"
        echo "  debug|dev  - Debug build (default)"
        echo "  release    - Release build"
        echo "  test       - Run tests"
        echo "  check      - Run cargo check"
        echo "  clippy     - Run clippy"
        echo "  fmt        - Run rustfmt"
        echo "  audit      - Run security audit"
        echo "  bench      - Run benchmarks"
        echo "  doc        - Build and open documentation"
        echo "  clean      - Clean build artifacts"
        echo "  all        - Run full validation pipeline"
        echo "  install    - Install locally"
        exit 1
        ;;
esac

echo -e "${GREEN}Build completed successfully!${NC}"