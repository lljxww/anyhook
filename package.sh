#!/usr/bin/env bash
set -e

# Build the release binary for the current architecture
echo "Building release binary..."
cargo build --release -p anyhook-cli

# Create dist directory
echo "Creating dist directory..."
rm -rf dist/
mkdir -p dist/anyhook

# Copy files
echo "Copying files..."
cp target/release/anyhook dist/anyhook/
cp anyhook.yaml dist/anyhook/anyhook.yaml.sample
cp README.md dist/anyhook/
cp LICENSE dist/anyhook/

# Compress based on OS
OS=$(uname -s)
ARCH=$(uname -m)
VERSION=$(grep -m1 version crates/anyhook-cli/Cargo.toml | cut -d '"' -f2)
ARCHIVE_NAME="anyhook-v${VERSION}-${OS}-${ARCH}"

cd dist

if [[ "$OS" == *"MINGW"* ]] || [[ "$OS" == *"CYGWIN"* ]]; then
    # Windows
    zip -r "${ARCHIVE_NAME}.zip" anyhook/
    echo "Successfully packaged to dist/${ARCHIVE_NAME}.zip"
else
    # Linux / macOS
    tar -czvf "${ARCHIVE_NAME}.tar.gz" anyhook/
    echo "Successfully packaged to dist/${ARCHIVE_NAME}.tar.gz"
fi

# If cargo-deb is installed and we're on Linux, build debian package
if [[ "$OS" == "Linux" ]]; then
    if command -v cargo-deb &> /dev/null; then
        echo "cargo-deb found. Building .deb package..."
        cd ..
        cargo deb -p anyhook-cli
        cp target/debian/*.deb dist/
        echo "Successfully packaged .deb to dist/"
    else
        echo "cargo-deb not found. Skipping .deb generation. Run 'cargo install cargo-deb' to enable."
    fi
fi

echo "Packaging complete!"
