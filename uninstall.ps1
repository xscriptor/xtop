# xtop uninstaller for Windows

$ErrorActionPreference = "Stop"

$AppName = "xtop"
$InstallDir = "$env:USERPROFILE\.cargo\bin"
$BinaryPath = "$InstallDir\$AppName.exe"

Write-Host "Uninstalling $AppName..." -ForegroundColor Green

if (Test-Path $BinaryPath) {
    Remove-Item -Force $BinaryPath
    Write-Host "$AppName removed successfully." -ForegroundColor Green
} else {
    Write-Host "$AppName not found in $InstallDir." -ForegroundColor Yellow
}
