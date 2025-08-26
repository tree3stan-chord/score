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

# Package information
PACKAGE = score
VERSION = 1.0.0

# Distribution and packaging targets
dist: clean
	@echo "📦 Creating distribution package..."
	tar czf $(PACKAGE)-$(VERSION).tar.gz \
		--exclude='.git*' \
		--exclude='target' \
		--exclude='*.json' \
		--exclude='test_*' \
		--exclude='*.musicxml' \
		--transform 's,^,$(PACKAGE)-$(VERSION)/,' \
		src/ Cargo.toml Cargo.lock Makefile README.md LICENSE \
		score.spec PKGBUILD *.md scripts/

rpm: dist
	@echo "🔴 Building RPM package..."
	@command -v rpmbuild >/dev/null 2>&1 || (echo "rpmbuild not available - install rpm-build package" && exit 1)
	mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
	cp $(PACKAGE)-$(VERSION).tar.gz ~/rpmbuild/SOURCES/
	cp $(PACKAGE).spec ~/rpmbuild/SPECS/
	rpmbuild -ba ~/rpmbuild/SPECS/$(PACKAGE).spec
	@echo "RPM packages created in ~/rpmbuild/RPMS/"

aur: dist
	@echo "🔵 Preparing AUR package..."
	@echo "Creating AUR directory..."
	rm -rf score-aur
	mkdir -p score-aur
	cp PKGBUILD score-aur/
	cp $(PACKAGE)-$(VERSION).tar.gz score-aur/
	cd score-aur && makepkg --printsrcinfo > .SRCINFO
	@echo "AUR package prepared in score-aur/"
	@echo "Upload contents to AUR git repository"

dev-install: release
	@echo "🔧 Installing SCORE for development..."
	mkdir -p ~/.local/bin
	cp target/release/score ~/.local/bin/
	@echo "SCORE installed to ~/.local/bin/score"
	@echo "Make sure ~/.local/bin is in your PATH"

uninstall:
	@echo "🗑️ Uninstalling SCORE..."
	@rm -f ~/.local/bin/score 2>/dev/null || true
	@rm -f /usr/local/bin/score 2>/dev/null || true  
	@cargo uninstall score 2>/dev/null || true
	@echo "Uninstall complete!"

smoke-test: build
	@echo "💨 Running smoke tests..."
	@echo "Testing binary execution..."
	@timeout 5s ./target/debug/score || test $$? -eq 124 && echo "✓ Binary runs (timed out as expected)"
	@echo "Testing help system..."
	@timeout 2s echo -e "\nq" | ./target/debug/score || test $$? -eq 124 && echo "✓ Interface loads"
	@echo "All smoke tests passed!"

package-info:
	@echo "📋 Package Information:"
	@echo "  Name:    $(PACKAGE)"
	@echo "  Version: $(VERSION)"
	@echo "  Binary:  target/release/$(PACKAGE)"

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
	@echo "  make smoke-test - Basic functionality tests"
	@echo ""
	@echo "Release:"
	@echo "  make release   - Build optimized release"
	@echo "  make install   - Install to system"
	@echo "  make dev-install - Install to ~/.local/bin"
	@echo ""
	@echo "Packaging:"
	@echo "  make dist      - Create source distribution"
	@echo "  make rpm       - Build RPM package"
	@echo "  make aur       - Prepare AUR package"
	@echo "  make package-info - Show package details"
	@echo ""
	@echo "Utilities:"
	@echo "  make clean     - Clean build artifacts"
	@echo "  make test-score - Create sample score for testing"
	@echo "  make uninstall - Remove installed SCORE"
	@echo "  make help      - Show this help"
	@echo ""
	@echo "Quick Start: make run"