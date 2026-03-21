# wux Windows Installer
param(
    [switch]$Force
)

$ErrorActionPreference = "Stop"

Write-Host "Installing wux..." -ForegroundColor Cyan

$RepoRoot = Split-Path -Parent $PSScriptRoot
$InstallDir = "$env:USERPROFILE\.wux\bin"
$BinaryPath = "$InstallDir\wux.exe"

if ((Test-Path $BinaryPath) -and -not $Force) {
    Write-Host "wux is already installed. Reinstalling..." -ForegroundColor Yellow
    $Force = $true
}

Push-Location $RepoRoot

Write-Host "Building release binary..." -ForegroundColor Yellow
cargo build --release

Pop-Location

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$RunningWux = Get-Process -Name "wux" -ErrorAction SilentlyContinue | Where-Object {
    $_.Path -and $_.Path -eq $BinaryPath
}
if ($RunningWux) {
    Write-Host "Stopping running wux..." -ForegroundColor Yellow
    $RunningWux | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 500
}

Copy-Item "$RepoRoot\target\release\wux.exe" $BinaryPath -Force
Write-Host "Copied wux.exe to $InstallDir" -ForegroundColor Green

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable(
        "Path",
        "$UserPath;$InstallDir",
        "User"
    )
    Write-Host "Added $InstallDir to PATH" -ForegroundColor Green
    Write-Host "NOTE: You may need to restart your terminal for changes to take effect." -ForegroundColor Yellow
}

$ConfigDir = "$env:APPDATA\wux"
$ConfigPath = "$ConfigDir\wux.toml"

if (-not (Test-Path $ConfigPath)) {
    if (-not (Test-Path $ConfigDir)) {
        New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null
    }

    @"
# wux configuration file

[settings]
color = true

[commands.free]
safe = true

[commands.nuke]
safe = false
"@ | Out-File -FilePath $ConfigPath -Encoding UTF8

    Write-Host "Created config file at $ConfigPath" -ForegroundColor Green
}

Write-Host ""
Write-Host "wux installed successfully!" -ForegroundColor Green
Write-Host "Open a new terminal and run 'wux --version' to confirm." -ForegroundColor Cyan
