#!/bin/bash
# Development helper script for SCORE

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[SCORE]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SCORE]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[SCORE]${NC} $1"
}

print_error() {
    echo -e "${RED}[SCORE]${NC} $1"
}

# Main development commands
case "${1:-help}" in
    "quick"|"q")
        print_status "🚀 Quick build and run..."
        cargo run
        ;;
    "build"|"b")
        print_status "🔨 Building SCORE..."
        cargo build
        print_success "Build complete!"
        ;;
    "release"|"r")
        print_status "🚀 Building release..."
        cargo build --release
        print_success "Release build complete!"
        ;;
    "clean"|"c")
        print_status "🧹 Cleaning..."
        cargo clean
        rm -f score.json test-score.json
        print_success "Clean complete!"
        ;;
    "test"|"t")
        print_status "🧪 Creating test score..."
        cat > test-score.json << 'EOF'
{
  "staves": [
    {
      "notes": [
        {"pitch": "C4", "duration": "Quarter", "accidental": "Natural", "position": 0},
        {"pitch": "E4", "duration": "Half", "accidental": "Natural", "position": 2},
        {"pitch": "G4", "duration": "Quarter", "accidental": "Sharp", "position": 4}
      ],
      "cursor_position": 5
    },
    {
      "notes": [
        {"pitch": "A4", "duration": "Whole", "accidental": "Natural", "position": 0},
        {"pitch": "F4", "duration": "Eighth", "accidental": "Flat", "position": 1}
      ],
      "cursor_position": 2
    }
  ]
}
EOF
        print_success "Test score created! Load with 'l' key in the app"
        cargo run
        ;;
    "check"|"ch")
        print_status "🔍 Running checks..."
        cargo check
        print_status "🎨 Formatting..."
        cargo fmt
        print_status "📝 Linting..."
        cargo clippy
        print_success "All checks passed!"
        ;;
    "watch"|"w")
        print_status "👀 Watching for changes..."
        if command -v cargo-watch >/dev/null 2>&1; then
            cargo watch -x run
        else
            print_warning "cargo-watch not installed. Install with: cargo install cargo-watch"
            print_status "Falling back to manual run..."
            cargo run
        fi
        ;;
    "install"|"i")
        print_status "📦 Installing SCORE..."
        cargo install --path .
        print_success "SCORE installed!"
        ;;
    "help"|"h"|*)
        echo -e "${BLUE}SCORE Development Helper${NC}"
        echo "========================"
        echo ""
        echo "Usage: ./scripts/dev.sh [command]"
        echo ""
        echo "Commands:"
        echo "  quick, q     - Quick build and run"
        echo "  build, b     - Build debug version"
        echo "  release, r   - Build release version"
        echo "  clean, c     - Clean build artifacts"
        echo "  test, t      - Create test score and run"
        echo "  check, ch    - Run all code checks"
        echo "  watch, w     - Watch for changes and rebuild"
        echo "  install, i   - Install to system"
        echo "  help, h      - Show this help"
        echo ""
        echo "Examples:"
        echo "  ./scripts/dev.sh quick    # Fast development cycle"
        echo "  ./scripts/dev.sh test     # Test with sample data"
        echo "  ./scripts/dev.sh check    # Code quality check"
        echo ""
        ;;
esac