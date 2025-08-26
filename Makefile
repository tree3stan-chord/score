# SCORE CLI Engraver Build System
.PHONY: all build run clean release test install dev watch lint fix

# Default target
all: build

# Build in debug mode
build:
	@echo "🔨 Building SCORE..."
	@cargo build

# Build in release mode  
release:
	@echo "🚀 Building SCORE (release)..."
	@cargo build --release

# Run the application
run: build
	@echo "🎼 Starting SCORE..."
	@cargo run

# Quick run without build check
quick:
	@echo "⚡ Quick start SCORE..."
	@cargo run

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean
	@rm -f score.json

# Install locally
install: release
	@echo "📦 Installing SCORE..."
	@cargo install --path .

# Development mode with watch
dev:
	@echo "👀 Starting development mode..."
	@echo "Use 'cargo watch -x run' if you have cargo-watch installed"
	@cargo run

# Check for issues without building
check:
	@echo "🔍 Checking code..."
	@cargo check

# Fix common issues
fix:
	@echo "🔧 Fixing common issues..."
	@cargo fix --allow-dirty --allow-staged

# Format code
fmt:
	@echo "💅 Formatting code..."
	@cargo fmt

# Run linter
lint:
	@echo "📝 Running clippy..."
	@cargo clippy

# Full check (format + lint + build)
verify: fmt lint build
	@echo "✅ All checks passed!"

# Create a test score and run
test-score: build
	@echo "🧪 Creating test score..."
	@echo '{"staves":[{"notes":[{"pitch":"C4","duration":"Quarter","accidental":"Natural","position":0},{"pitch":"E4","duration":"Quarter","accidental":"Natural","position":1},{"pitch":"G4","duration":"Quarter","accidental":"Natural","position":2}],"cursor_position":3}]}' > test-score.json
	@echo "Test score created as test-score.json"
	@echo "Run with: make run"

# Show help
help:
	@echo "SCORE CLI Engraver - Build System"
	@echo "=================================="
	@echo ""
	@echo "Development:"
	@echo "  make build     - Build in debug mode"
	@echo "  make run       - Build and run SCORE"
	@echo "  make quick     - Quick run without build check"
	@echo "  make dev       - Development mode"
	@echo ""
	@echo "Quality:"
	@echo "  make check     - Check code without building"
	@echo "  make lint      - Run clippy linter"
	@echo "  make fmt       - Format code"
	@echo "  make fix       - Fix common issues"
	@echo "  make verify    - Full verification (fmt + lint + build)"
	@echo ""
	@echo "Release:"
	@echo "  make release   - Build optimized release"
	@echo "  make install   - Install to system"
	@echo ""
	@echo "Utilities:"
	@echo "  make clean     - Clean build artifacts"
	@echo "  make test-score - Create sample score for testing"
	@echo "  make help      - Show this help"
	@echo ""
	@echo "Quick Start: make run"