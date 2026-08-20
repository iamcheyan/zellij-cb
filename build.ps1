$ErrorActionPreference = "Stop"

Set-Location $PSScriptRoot

$target = "wasm32-wasip1"
$outDir = Join-Path "target" "$target\release"
$wasm = "zellij-cb.wasm"

$installedTargets = rustup target list --installed
if (-not ($installedTargets -split "\r?\n" -contains $target)) {
    Write-Host "Installing target $target..."
    rustup target add $target
}

Write-Host "Building $wasm..."
cargo build --target $target --release

$configDir = if ($env:ZELLIJ_CONFIG_DIR) {
    $env:ZELLIJ_CONFIG_DIR
} elseif ($env:APPDATA) {
    Join-Path $env:APPDATA "zellij"
} else {
    Join-Path $HOME ".config\zellij"
}
$destinationDir = Join-Path $configDir "plugins"
New-Item -ItemType Directory -Force -Path $destinationDir | Out-Null
$destination = Join-Path $destinationDir $wasm
Copy-Item (Join-Path $outDir $wasm) $destination -Force
Write-Host "Installed: $destination"
