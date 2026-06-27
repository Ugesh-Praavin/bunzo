$InstallDir = "$env:LOCALAPPDATA\Bunzo"
$BaseUrl = "https://downloads.sourceforge.net/project/bunzo"
$ZipPath = "$env:TEMP\bunzo-windows-x86_64.zip"
$ExtractPath = "$env:TEMP\bunzo-extract"

Write-Host "Installing Bunzo..."

# Download
Write-Host "Downloading bunzo..."
Invoke-WebRequest -Uri "$BaseUrl/bunzo-windows-x86_64.zip" -OutFile $ZipPath

# Extract
Expand-Archive -Path $ZipPath -DestinationPath $ExtractPath -Force

# Create install dir
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

# Copy exe and runtime
Copy-Item "$ExtractPath\bunzo.exe" "$InstallDir\bunzo.exe" -Force
Copy-Item "$ExtractPath\runtime" "$InstallDir\runtime" -Recurse -Force

# Verify copy
if (-Not (Test-Path "$InstallDir\bunzo.exe")) {
    Write-Host "Error: Installation failed!" -ForegroundColor Red
    exit 1
}

# Add to PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
    Write-Host "Added to PATH." -ForegroundColor Green
} else {
    Write-Host "Already in PATH, skipping." -ForegroundColor Yellow
}

# Cleanup
Remove-Item $ZipPath -Force
Remove-Item $ExtractPath -Recurse -Force

Write-Host "Bunzo installed successfully!" -ForegroundColor Green
Write-Host "Restart your terminal, then run: bunzo --help"