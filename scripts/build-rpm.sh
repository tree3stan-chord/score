#!/usr/bin/env bash
# SCORE RPM Build Script
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
PACKAGE="score"
VERSION="1.0.0"

echo "🎼 Building SCORE RPM Package"
echo "==============================="

# Change to project directory
cd "$PROJECT_DIR"

# Ensure we have a clean, release build
echo "▶ Building SCORE in release mode..."
make clean
make release

# Run basic tests
echo "▶ Running smoke tests..."
make smoke-test

# Create source distribution
echo "▶ Creating source distribution..."
make dist

# Verify the tarball was created
if [[ ! -f "${PACKAGE}-${VERSION}.tar.gz" ]]; then
    echo "❌ Source tarball not found!"
    exit 1
fi

# Setup RPM build environment
echo "▶ Setting up RPM build environment..."
mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Copy files to RPM build directories
echo "▶ Copying files for RPM build..."
cp "${PACKAGE}-${VERSION}.tar.gz" ~/rpmbuild/SOURCES/
cp "${PACKAGE}.spec" ~/rpmbuild/SPECS/

# Build the RPM packages
echo "▶ Building RPM packages..."
rpmbuild -ba ~/rpmbuild/SPECS/${PACKAGE}.spec

# Verify the packages were built
echo "▶ Verifying RPM packages..."
if ls ~/rpmbuild/RPMS/*/score-*.rpm 1> /dev/null 2>&1; then
    echo "✅ Binary RPM package built successfully!"
    ls -la ~/rpmbuild/RPMS/*/score-*.rpm
else
    echo "❌ Failed to build binary RPM package"
    exit 1
fi

if ls ~/rpmbuild/SRPMS/score-*.src.rpm 1> /dev/null 2>&1; then
    echo "✅ Source RPM package built successfully!"
    ls -la ~/rpmbuild/SRPMS/score-*.src.rpm
else
    echo "❌ Failed to build source RPM package"
    exit 1
fi

# Test install the package (if running as root or with sudo)
if [[ $EUID -eq 0 ]] || sudo -n true 2>/dev/null; then
    echo "▶ Testing RPM installation..."
    
    # Find the binary RPM
    RPM_FILE=$(find ~/rpmbuild/RPMS -name "score-*.rpm" -not -name "*.src.rpm" | head -1)
    
    if [[ -n "$RPM_FILE" ]]; then
        # Test the RPM
        echo "Testing RPM file: $RPM_FILE"
        rpm -qip "$RPM_FILE"
        
        # Optional: Install and test
        read -p "Do you want to install and test the RPM? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            sudo rpm -i "$RPM_FILE" || sudo rpm -U "$RPM_FILE"
            echo "✅ RPM installed successfully!"
            
            # Test the installed binary
            echo "▶ Testing installed binary..."
            timeout 5s score || test $? -eq 124 && echo "✅ Installed binary works!"
        fi
    fi
else
    echo "⚠️  Skipping RPM installation test (no sudo access)"
fi

echo ""
echo "🎉 RPM Build Complete!"
echo "======================"
echo "Binary RPM:  ~/rpmbuild/RPMS/x86_64/score-${VERSION}-1.el*.x86_64.rpm"
echo "Source RPM:  ~/rpmbuild/SRPMS/score-${VERSION}-1.el*.src.rpm"
echo ""
echo "To install: sudo rpm -i ~/rpmbuild/RPMS/x86_64/score-${VERSION}-*.rpm"
echo "To upload to repository: copy to repos.musicsian.com/RPMS/"