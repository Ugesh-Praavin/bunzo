$InstallDir = "$env:LOCALAPPDATA\Bunzo"
$BaseUrl = "https://downloads.sourceforge.net/project/bunzo"
$ZipPath = "$env:TEMP\bunzo-windows-x86_64.zip"
$ExtractPath = "$env:TEMP\bunzo-extract"

Write-Host "Installing Bunzo..."

# Download
Write-Host "Downloading bunzo..."
try {
    curl.exe -fsSL "$BaseUrl/bunzo-windows-x86_64.zip" -o $ZipPath
} catch {
    Write-Host "Error: Download failed! Check your internet connection." -ForegroundColor Red
    exit 1
}

# Extract
Write-Host "Extracting..."
mkdir -Force $ExtractPath | Out-Null
Expand-Archive -Path $ZipPath -DestinationPath $ExtractPath -Force

# Check exe exists in zip
if (-Not (Test-Path "$ExtractPath\bunzo.exe")) {
    Write-Host "Error: bunzo.exe not found in archive!" -ForegroundColor Red
    exit 1
}

# Create install dir
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

# Copy exe
Copy-Item "$ExtractPath\bunzo.exe" "$InstallDir\bunzo.exe" -Force

# Copy runtime only if it exists
if (Test-Path "$ExtractPath\runtime") {
    New-Item -ItemType Directory -Force -Path "$InstallDir\runtime" | Out-Null
    Copy-Item "$ExtractPath\runtime\*" "$InstallDir\runtime\" -Recurse -Force
    Write-Host "Runtime copied." -ForegroundColor Green
}

# Verify install
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

Write-Host "Done! Restart your terminal, then run: bunzo --help" -ForegroundColor Green