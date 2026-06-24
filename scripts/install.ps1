# Bunzo Windows Installer

$InstallDir = "$env:LOCALAPPDATA\Bunzo"

New-Item -ItemType Directory -Force -Path $InstallDir

Copy-Item bunzo.exe $InstallDir -Force

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")

if ($userPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable(
        "Path",
        "$userPath;$InstallDir",
        "User"
    )
}

Write-Host "Bunzo installed successfully!"
Write-Host "Restart your terminal, then run: bunzo --help"