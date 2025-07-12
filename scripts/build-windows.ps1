# Windows MSI Build Script for Cosmic Noise
# This script prepares the application for MSI packaging

Write-Host "Building Cosmic Noise for Windows..." -ForegroundColor Green

# Create staging directory
$stagingDir = "dist"
if (Test-Path $stagingDir) {
    Remove-Item -Recurse -Force $stagingDir
}
New-Item -ItemType Directory -Path $stagingDir

# Copy executable
Copy-Item "target\x86_64-pc-windows-msvc\release\cosmic_noise.exe" "$stagingDir\cosmic_noise.exe"

# Copy resources
Copy-Item -Recurse "assets" "$stagingDir\assets"
Copy-Item -Recurse "i18n" "$stagingDir\i18n"
Copy-Item -Recurse "resources" "$stagingDir\resources"

Write-Host "Staging complete. Files ready for MSI packaging." -ForegroundColor Green
Write-Host "Run: cargo wix --release --path dist" -ForegroundColor Yellow 