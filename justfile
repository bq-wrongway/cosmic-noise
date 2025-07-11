# Justfile for Flatpak-only builds of Cosmic Noise

# Path to the Flatpak manifest
manifest := 'io.github.bqwrongway.CosmicNoise.yaml'
app_id := 'io.github.bqwrongway.CosmicNoise'

# Build the Flatpak package
flatpak-build:
    flatpak-builder --force-clean build-dir {{manifest}}

# Install the Flatpak locally for testing
flatpak-install:
    flatpak-builder --user --install --force-clean build-dir {{manifest}}

# Run the app via Flatpak
flatpak-run:
    flatpak run {{app_id}}

# Clean Flatpak build artifacts
flatpak-clean:
    rm -rf build-dir

# Export a Flatpak bundle for distribution
flatpak-bundle:
    flatpak-builder --repo=repo --force-clean build-dir {{manifest}}
    flatpak build-bundle repo {{app_id}}.flatpak {{app_id}} stable

# Development commands
dev:
    cargo run

dev-debug:
    RUST_LOG=debug cargo run

build:
    cargo build --release

test:
    cargo test

check:
    cargo check

clippy:
    cargo clippy

fmt:
    cargo fmt

fmt-check:
    cargo fmt -- --check

# Clean everything
clean: flatpak-clean
    cargo clean
    rm -rf vendor/
    rm -rf repo/
    rm -f *.flatpak

# Setup for development
setup:
    cargo vendor
    echo "Development environment ready!" 