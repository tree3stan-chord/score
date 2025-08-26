# Justfile for SCORE CLI Engraver
# Install just with: cargo install just
# Run with: just <command>

# Default recipe
default: run

# Quick build and run
run:
    @echo "🎼 Starting SCORE..."
    cargo run

# Build debug version  
build:
    @echo "🔨 Building..."
    cargo build

# Build release version
release:
    @echo "🚀 Building release..."
    cargo build --release

# Clean everything
clean:
    @echo "🧹 Cleaning..."
    cargo clean
    rm -f score.json test-score.json

# Format and lint
check:
    @echo "💅 Formatting..."
    cargo fmt
    @echo "📝 Linting..."  
    cargo clippy
    @echo "🔍 Checking..."
    cargo check

# Create test score and run
test:
    @echo "🧪 Creating test score..."
    echo '{"staves":[{"notes":[{"pitch":"C4","duration":"Quarter","accidental":"Natural","position":0},{"pitch":"E4","duration":"Half","accidental":"Natural","position":2},{"pitch":"G4","duration":"Quarter","accidental":"Sharp","position":4}],"cursor_position":5},{"notes":[{"pitch":"A4","duration":"Whole","accidental":"Natural","position":0}],"cursor_position":1}]}' > test-score.json
    @echo "Test score ready! Use 'l' to load in app"
    just run

# Install globally
install: release
    @echo "📦 Installing..."
    cargo install --path .

# Watch for changes (requires cargo-watch)
watch:
    cargo watch -x run

# Show all commands
list:
    @just --list