# Maintainer: mfw <espadon@outlook.com>

pkgname=score
pkgver=1.0.0
pkgrel=1
pkgdesc="Professional CLI Music Notation System - A colorized, feature-rich TUI score engraver"
arch=('x86_64')
url="https://github.com/musicsian-com/score"
license=('MIT')
depends=('alsa-lib')
makedepends=('rust' 'cargo' 'gcc')
source=("$pkgname-$pkgver.tar.gz")
sha256sums=('SKIP')  # Will be updated when creating actual package

build() {
    cd "$pkgname-$pkgver"
    
    # Build in release mode for performance
    cargo build --release --locked --target-dir=target
}

check() {
    cd "$pkgname-$pkgver"
    
    # Run basic tests
    cargo test --release --target-dir=target
    
    # Test binary execution (with timeout since it's an interactive app)
    timeout 5s ./target/release/score || test $? -eq 124
}

package() {
    cd "$pkgname-$pkgver"
    
    # Install the binary
    install -Dm755 target/release/score "$pkgdir/usr/bin/score"
    
    # Install license
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    
    # Install documentation
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
    install -Dm644 COLORIZATION_SUMMARY.md "$pkgdir/usr/share/doc/$pkgname/COLORIZATION_SUMMARY.md"
    install -Dm644 COLORS_COMPLETE.md "$pkgdir/usr/share/doc/$pkgname/COLORS_COMPLETE.md" 
    install -Dm644 SIGNATURE_CHANGES_COMPLETE.md "$pkgdir/usr/share/doc/$pkgname/SIGNATURE_CHANGES_COMPLETE.md"
    install -Dm644 SCROLLING_COMPLETE.md "$pkgdir/usr/share/doc/$pkgname/SCROLLING_COMPLETE.md"
}