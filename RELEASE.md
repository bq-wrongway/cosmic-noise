# Release Guide

This document explains how to create releases for Cosmic Noise using GitHub Actions.

## Automated Releases

The project uses GitHub Actions to automatically build and package the application for multiple platforms when you push a version tag.

### Creating a Release

1. **Update Version**: Update the version in `Cargo.toml`
2. **Commit Changes**: Commit your changes
3. **Create Tag**: Create and push a version tag
   ```bash
   git tag v0.2.6
   git push origin v0.2.6
   ```

### What Gets Built

The workflow automatically creates:

- **macOS**: `.app` bundle with all resources included
- **Windows**: `.msi` installer with all resources included  
- **Linux**: `.flatpak` bundle (using existing Flatpak configuration)

### Workflow Details

The workflow runs on:
- `ubuntu-latest` (Linux/Flatpak)
- `windows-latest` (Windows MSI)
- `macos-latest` (macOS .app)

### Resources Included

All packages include:
- Application executable
- `assets/` directory (fonts, icons)
- `i18n/` directory (localization files)
- `resources/` directory (sound files)

### Manual Building

If you need to build locally:

#### macOS
```bash
cargo install cargo-bundle
cargo bundle --release
```

#### Windows
```bash
cargo install cargo-wix
cargo wix --release
```

#### Linux
```bash
flatpak-builder build-dir io.github.bq-wrongway.CosmicNoise.yaml
```

### Release Artifacts

After a successful workflow run, you'll find:
- `CosmicNoise.app` (macOS)
- `CosmicNoise.msi` (Windows)
- `cosmic-noise.flatpak` (Linux)

These are automatically uploaded to GitHub Releases when you push a version tag. 