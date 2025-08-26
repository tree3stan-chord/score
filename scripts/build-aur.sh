#!/usr/bin/env bash
# SCORE AUR Package Build Script
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
PACKAGE="score"
VERSION="1.0.0"

echo "🎼 Building SCORE AUR Package"
echo "=============================="

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

# Calculate SHA256 sum for the source tarball
echo "▶ Calculating SHA256 checksum..."
SHA256=$(sha256sum "${PACKAGE}-${VERSION}.tar.gz" | cut -d' ' -f1)
echo "SHA256: $SHA256"

# Update PKGBUILD with the correct checksum
echo "▶ Updating PKGBUILD with SHA256 checksum..."
sed -i "s/sha256sums=('SKIP')/sha256sums=('$SHA256')/" PKGBUILD

# Create AUR directory
echo "▶ Preparing AUR package directory..."
AUR_DIR="${PACKAGE}-aur"
rm -rf "$AUR_DIR"
mkdir -p "$AUR_DIR"

# Copy necessary files
cp PKGBUILD "$AUR_DIR/"
cp "${PACKAGE}-${VERSION}.tar.gz" "$AUR_DIR/"

# Generate .SRCINFO
echo "▶ Generating .SRCINFO..."
cd "$AUR_DIR"

# Check if makepkg is available
if ! command -v makepkg &> /dev/null; then
    echo "❌ makepkg not found! Please install base-devel package on Arch Linux."
    exit 1
fi

# Generate .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# Validate the PKGBUILD
echo "▶ Validating PKGBUILD..."
makepkg --check

# Optional: Test build the package
read -p "Do you want to test build the AUR package? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "▶ Test building package..."
    makepkg -s --noconfirm
    
    if ls ${PACKAGE}-${VERSION}-*.pkg.tar.* 1> /dev/null 2>&1; then
        echo "✅ Package built successfully!"
        ls -la ${PACKAGE}-${VERSION}-*.pkg.tar.*
        
        # Optional: Install and test
        read -p "Do you want to install and test the package? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            sudo pacman -U ${PACKAGE}-${VERSION}-*.pkg.tar.* --noconfirm
            echo "✅ Package installed successfully!"
            
            # Test the installed binary
            echo "▶ Testing installed binary..."
            timeout 5s score || test $? -eq 124 && echo "✅ Installed binary works!"
        fi
    else
        echo "❌ Package build failed!"
        exit 1
    fi
fi

cd "$PROJECT_DIR"

echo ""
echo "🎉 AUR Package Preparation Complete!"
echo "===================================="
echo "AUR Directory: $AUR_DIR/"
echo "Contents:"
echo "  - PKGBUILD (with correct SHA256)"
echo "  - .SRCINFO (package metadata)"
echo "  - ${PACKAGE}-${VERSION}.tar.gz (source)"
echo ""
echo "Next steps for AUR submission:"
echo "1. Clone the AUR repository:"
echo "   git clone ssh://aur@aur.archlinux.org/${PACKAGE}.git"
echo "2. Copy files to the cloned directory:"
echo "   cp $AUR_DIR/PKGBUILD $AUR_DIR/.SRCINFO aur-${PACKAGE}/"
echo "3. Commit and push:"
echo "   cd aur-${PACKAGE} && git add . && git commit -m 'Initial import' && git push"
echo ""
echo "SHA256 checksum: $SHA256"