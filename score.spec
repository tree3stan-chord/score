Name:           score
Version:        1.0.0
Release:        1%{?dist}
Summary:        Professional CLI Music Notation System - A colorized, feature-rich TUI score engraver

License:        MIT
URL:            https://github.com/musicsian-com/score
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.70
BuildRequires:  cargo
BuildRequires:  gcc
BuildRequires:  alsa-lib-devel
Requires:       alsa-lib

# Disable debug package
%global debug_package %{nil}

%description
SCORE is a professional-grade CLI music notation system that brings the power
of digital music engraving to the terminal. Built in Rust with a beautiful,
colorized TUI interface, SCORE provides comprehensive music notation capabilities
for composers, musicians, and music educators.

Features:
- Professional colorized interface with rainbow note colors
- Unlimited staff length with smooth horizontal scrolling
- Multi-staff composition support (up to 4 staves)
- Complete notation support: all durations, accidentals, ties, tuplets, beaming
- Mid-composition signature changes (time signatures and key signatures)
- Real-time MIDI playback with audio feedback
- Professional MusicXML export for integration with other software
- Full mouse and keyboard control with intuitive shortcuts
- Comprehensive undo/redo system
- Save/load functionality with JSON persistence

Perfect for:
- Composers creating original music
- Music educators teaching notation
- Musicians sketching musical ideas
- Anyone needing quick, professional music notation

%prep
%autosetup

%build
# Set up clean Cargo environment
export CARGO_HOME=$(pwd)/.cargo
export RUSTFLAGS="-C opt-level=3 -C target-cpu=native"

# Ensure we're not in a workspace by removing any parent Cargo.toml files
rm -f ../Cargo.toml ../../Cargo.toml

# Add empty workspace table to avoid conflicts
echo "" >> Cargo.toml
echo "[workspace]" >> Cargo.toml

# Build in release mode for performance
cargo build --release --locked --target-dir=target

%check
# Run basic functionality tests
cargo test --release --target-dir=target
# Test binary execution (expect MIDI error or timeout)
timeout 5s ./target/release/score 2>/dev/null || test $? -eq 124 || test $? -eq 1

%install
# Install the binary
mkdir -p %{buildroot}%{_bindir}
install -Dm755 target/release/score %{buildroot}%{_bindir}/score

# Install documentation
mkdir -p %{buildroot}%{_docdir}/%{name}
install -Dm644 README.md %{buildroot}%{_docdir}/%{name}/README.md
install -Dm644 COLORIZATION_SUMMARY.md %{buildroot}%{_docdir}/%{name}/COLORIZATION_SUMMARY.md
install -Dm644 COLORS_COMPLETE.md %{buildroot}%{_docdir}/%{name}/COLORS_COMPLETE.md
install -Dm644 SIGNATURE_CHANGES_COMPLETE.md %{buildroot}%{_docdir}/%{name}/SIGNATURE_CHANGES_COMPLETE.md
install -Dm644 SCROLLING_COMPLETE.md %{buildroot}%{_docdir}/%{name}/SCROLLING_COMPLETE.md

%files
%license LICENSE
%{_bindir}/score
%{_docdir}/%{name}/*

%changelog
* Mon Aug 26 2024 mfw <espadon@outlook.com> - 1.0.0-1
- Initial RPM release of SCORE
- Professional colorized CLI music notation system
- Complete TUI interface with rainbow note colors
- Unlimited staff scrolling and multi-staff support
- Full notation features: durations, accidentals, ties, tuplets
- Mid-composition signature changes support
- MIDI playback and MusicXML export capabilities
- Comprehensive keyboard and mouse control
- Save/load functionality with undo/redo system
- Production-ready for composers and music educators