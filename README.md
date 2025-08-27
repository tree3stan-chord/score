# 🎼 SCORE - Professional CLI Music Notation System

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-red.svg)](https://www.rust-lang.org/)

**SCORE** is a revolutionary CLI music notation system that brings professional music engraving to your terminal. Built in Rust with a stunning colorized TUI interface, SCORE transforms music composition from complex software to intuitive, keyboard-driven creativity.

## ✨ Features

### 🌈 **Beautiful Colorized Interface**
- **Rainbow note colors**: Each pitch (C/D/E/F/G/A/B) has unique, vibrant colors
- **Professional appearance**: Subtle staff lines, bright active elements
- **Visual hierarchy**: Color-coded signatures, cursor highlighting, scroll indicators
- **Accessibility**: High contrast ratios, non-color cues preserved

### 🎵 **Professional Music Notation**
- **All standard durations**: Whole, half, quarter, eighth, sixteenth notes
- **Complete accidental support**: Sharps, flats, naturals with color coding
- **Advanced notation**: Ties, tuplets, beaming with visual grouping
- **Multi-staff composition**: Up to 4 staves for quartet writing
- **Unlimited length**: Horizontal scrolling for full songs and pieces

### 📊 **Dynamic Signature Changes**
- **Mid-composition changes**: Insert time and key signature changes anywhere
- **Visual indicators**: Color-coded signature markers on staff
- **Smart rendering**: Automatic measure boundaries and proper MusicXML export
- **Easy controls**: `Shift+T` for time signatures, `Shift+K` for key signatures

### 🎮 **Intuitive Controls**
- **Keyboard-first design**: All functions accessible via shortcuts
- **Mouse support**: Point-and-click editing capabilities
- **Smart navigation**: Home/End jumping, Page Up/Down scrolling
- **Help system**: Comprehensive, colorized control reference

### 🔊 **Audio & Export**
- **Real-time MIDI playback**: Hear your compositions as you write
- **Professional MusicXML export**: Compatible with MuseScore, Finale, Sibelius
- **Save/Load system**: JSON persistence with undo/redo support

## 🚀 Quick Start

### Installation

#### From AUR (Arch Linux)
```bash
yay -S score
```

#### From RPM (RHEL/Fedora/CentOS)
```bash
# Add the musicsian repository
curl -s https://repos.musicsian.com/install-repo.sh | sudo bash

# Install SCORE
sudo dnf install score
```

#### From Source
```bash
git clone https://github.com/musicsian-com/score
cd score
make install
```

### Usage

```bash
# Start SCORE
score

# Quick workflow:
# 1. Configure time/key signatures (F1)
# 2. Enter notes (c/d/e/f/g/a/b)
# 3. Change durations (SPACE)
# 4. Add accidentals (#)
# 5. Export to MusicXML (x)
# 6. Play with MIDI (p)
```

## 🎯 Use Cases

### 🎼 **For Composers**
- Sketch musical ideas quickly
- Write full songs with unlimited scrolling
- Export to professional notation software
- Test ideas with immediate MIDI playback

### 🏫 **For Educators**
- Teach music notation visually
- Demonstrate rhythm and pitch relationships
- Create exercises and examples
- Show signature changes and their effects

### 🎸 **For Musicians**
- Notate chord progressions
- Write lead sheets and arrangements
- Share musical ideas in standard notation
- Practice sight-reading with colorized pitches

### 💻 **For Developers**
- Terminal-friendly music notation
- Scriptable composition workflows
- Integration with other CLI tools
- Version control friendly file format

## 🎨 Interface Examples

### Main Composition View
```
🎼 Score - Staff 1/2 | 4/4 Time | C major Key | Current: 3/4 D major
────────────────────────────────────────────────────────────────────
     3/4              2♯                               
─────│─────────────────│───────────────────────────────────────────
     │     ●           │         ♪                    
─────│─────────────────│───────────────────────────────────────────
     │                 │                              
─────│─────────────────│───────────────────────────────────────────
     │                 │           ●                  
─────│─────────────────│───────────────────────────────────────────
     │     𝄽           │         𝄽       ║            ◀ ▶
─────│─────────────────│───────────────────────────────────────────
     │                 │                              
─────│─────────────────│───────────────────────────────────────────
           C                                          
```
*With rainbow colors for each note and element*

### Colorized Help System
```
Controls: ←→=cursor, ↑↓=staff, notes=c/d/e/f/g/a/b, SPACE=duration(quarter), 
#=accidental(natural), t=tie, Shift+T=time sig, Shift+K=key sig, 
F1=setup, p=play, x=export, q=quit
```

## 🛠️ Development

```bash
# Build and run
make run

# Development mode
make dev

# Run tests
make smoke-test

# Package for distribution
make dist      # Create source tarball
make rpm       # Build RPM package
make aur       # Prepare AUR package
```

## 📚 Documentation

- [**Colorization Guide**](COLORS_COMPLETE.md) - Complete color scheme documentation
- [**Signature Changes**](SIGNATURE_CHANGES_COMPLETE.md) - Mid-composition signature features
- [**Scrolling System**](SCROLLING_COMPLETE.md) - Unlimited length composition
- [**Build Instructions**](BUILD.md) - Development setup and compilation

## 🤝 Contributing

We welcome contributions! SCORE is built with:
- **Rust** with `ratatui` for the TUI interface
- **Crossterm** for terminal control
- **Serde** for JSON serialization
- **xml-rs** for MusicXML export
- **midir** for MIDI playback

## 📄 License

SCORE is released under the MIT License. See [LICENSE](LICENSE) for details.

## 🎵 Created by Musicians, for Musicians

SCORE bridges the gap between traditional music notation software and modern terminal-based workflows. Whether you're a composer, educator, or developer, SCORE provides the tools you need for professional music notation in a beautiful, efficient CLI environment.

**Start composing beautiful music today!** 🌈✨