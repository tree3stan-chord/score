#!/usr/bin/env bash
# SCORE Repository Deployment Script
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
PACKAGE="score"
VERSION="1.0.0"
REPO_PATH="/home/espadon/src/repos-musicsian-com"

echo "🎼 Deploying SCORE to Musicsian Repository"
echo "==========================================="

# Change to project directory
cd "$PROJECT_DIR"

# Ensure RPM is built
if [[ ! -f ~/rpmbuild/RPMS/x86_64/${PACKAGE}-${VERSION}-*.rpm ]]; then
    echo "▶ RPM not found, building..."
    ./scripts/build-rpm.sh
fi

# Find the RPM files
RPM_FILE=$(find ~/rpmbuild/RPMS -name "${PACKAGE}-${VERSION}-*.rpm" -not -name "*.src.rpm" | head -1)
SRPM_FILE=$(find ~/rpmbuild/SRPMS -name "${PACKAGE}-${VERSION}-*.src.rpm" | head -1)

if [[ -z "$RPM_FILE" ]] || [[ -z "$SRPM_FILE" ]]; then
    echo "❌ RPM files not found!"
    exit 1
fi

echo "▶ Found packages:"
echo "  Binary RPM: $RPM_FILE"
echo "  Source RPM: $SRPM_FILE"

# Verify repository directory exists
if [[ ! -d "$REPO_PATH" ]]; then
    echo "❌ Repository directory not found: $REPO_PATH"
    echo "Please ensure repos-musicsian-com is available"
    exit 1
fi

# Copy RPM files to repository
echo "▶ Copying RPM files to repository..."
cp "$RPM_FILE" "$REPO_PATH/RPMS/"
cp "$SRPM_FILE" "$REPO_PATH/RPMS/"

echo "✅ RPM files copied to repository"

# Create HTML description file
echo "▶ Creating repository HTML page..."
cat > "$REPO_PATH/score.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>SCORE - Professional CLI Music Notation System</title>
    <style>
        body { font-family: system-ui, -apple-system, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }
        .header { background: linear-gradient(45deg, #667eea, #764ba2); color: white; padding: 20px; border-radius: 8px; }
        .feature { background: #f8f9fa; padding: 15px; margin: 10px 0; border-left: 4px solid #667eea; }
        .install { background: #2d3748; color: white; padding: 15px; border-radius: 4px; font-family: monospace; }
        .badge { background: #667eea; color: white; padding: 2px 8px; border-radius: 12px; font-size: 0.8em; }
    </style>
</head>
<body>
    <div class="header">
        <h1>🎼 SCORE - Professional CLI Music Notation System</h1>
        <p><span class="badge">v1.0.0</span> <span class="badge">CLI</span> <span class="badge">Music</span> <span class="badge">Rust</span></p>
    </div>

    <h2>Overview</h2>
    <p>SCORE is a revolutionary CLI music notation system that brings professional music engraving to your terminal. Built in Rust with a stunning colorized TUI interface, SCORE transforms music composition from complex software to intuitive, keyboard-driven creativity.</p>

    <div class="feature">
        <h3>🌈 Beautiful Colorized Interface</h3>
        <p>Rainbow note colors, professional appearance, visual hierarchy with color-coded signatures and cursor highlighting.</p>
    </div>

    <div class="feature">
        <h3>🎵 Professional Music Notation</h3>
        <p>All standard durations, complete accidental support, advanced notation with ties and tuplets, multi-staff composition support.</p>
    </div>

    <div class="feature">
        <h3>📊 Dynamic Signature Changes</h3>
        <p>Mid-composition time and key signature changes with visual indicators and smart MusicXML export.</p>
    </div>

    <div class="feature">
        <h3>🔊 Audio & Export</h3>
        <p>Real-time MIDI playback and professional MusicXML export compatible with major music software.</p>
    </div>

    <h2>Installation</h2>
    <div class="install">
# Add the musicsian repository
curl -s https://repos.musicsian.com/install-repo.sh | sudo bash

# Install SCORE
sudo dnf install score

# Start composing!
score
    </div>

    <h2>Quick Start</h2>
    <ul>
        <li><strong>F1</strong> - Configure time/key signatures</li>
        <li><strong>c/d/e/f/g/a/b</strong> - Enter notes (color-coded!)</li>
        <li><strong>SPACE</strong> - Change durations</li>
        <li><strong>#</strong> - Add accidentals</li>
        <li><strong>Shift+T/K</strong> - Insert signature changes</li>
        <li><strong>p</strong> - Play with MIDI</li>
        <li><strong>x</strong> - Export to MusicXML</li>
    </ul>

    <h2>Perfect For</h2>
    <ul>
        <li>🎼 <strong>Composers</strong> - Sketch ideas, write full songs</li>
        <li>🏫 <strong>Educators</strong> - Teach notation visually</li>
        <li>🎸 <strong>Musicians</strong> - Notate chord progressions, lead sheets</li>
        <li>💻 <strong>Developers</strong> - Terminal-friendly music workflows</li>
    </ul>

    <p><a href="https://github.com/musicsian-com/score">GitHub Repository</a> | <a href="https://repos.musicsian.com/">Back to Repository</a></p>
</body>
</html>
EOF

# Create .repo file
echo "▶ Creating repository configuration..."
cat > "$REPO_PATH/score.repo" << EOF
[musicsian-score]
name=Musicsian Repository - SCORE
baseurl=https://repos.musicsian.com
enabled=1
gpgcheck=1
gpgkey=https://repos.musicsian.com/RPM-GPG-KEY-musicsian
EOF

# Update main repository metadata
echo "▶ Updating repository metadata..."
cd "$REPO_PATH"
createrepo_c --update .

# Sign the repository metadata if GPG is available
if command -v gpg &> /dev/null; then
    echo "▶ Signing repository metadata..."
    gpg --detach-sign --armor repodata/repomd.xml
    echo "✅ Repository metadata signed"
else
    echo "⚠️  GPG not available, skipping signature"
fi

echo ""
echo "🎉 SCORE Successfully Deployed!"
echo "==============================="
echo "Repository files updated:"
echo "  - Binary RPM: RPMS/$(basename "$RPM_FILE")"
echo "  - Source RPM: RPMS/$(basename "$SRPM_FILE")"
echo "  - Description: score.html"
echo "  - Repo config: score.repo"
echo ""
echo "Users can now install with:"
echo "  curl -s https://repos.musicsian.com/install-repo.sh | sudo bash"
echo "  sudo dnf install score"
echo ""
echo "Next steps:"
echo "1. Deploy repository with: cd $REPO_PATH && ./deploy.sh"
echo "2. Announce on musicsian.com"
echo "3. Submit to AUR with ./scripts/build-aur.sh"