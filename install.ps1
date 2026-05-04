#Requires -Version 5.1

<#
.SYNOPSIS
    One-line installer for jjit on Windows
.DESCRIPTION
    Automatically detects platform, downloads latest binary from GitHub releases,
    and installs jjit to the user's local application directory.
#>

$ErrorActionPreference = "Stop"

$Repo = "LaneSun/jjit"
$InstallDir = "$env:LOCALAPPDATA\jjit"

function Write-Info($Message) {
    Write-Host $Message -ForegroundColor Cyan
}

function Write-Success($Message) {
    Write-Host $Message -ForegroundColor Green
}

function Write-Warn($Message) {
    Write-Host $Message -ForegroundColor Yellow
}

function Get-LatestVersion {
    try {
        $response = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -UseBasicParsing
        return $response.tag_name.TrimStart('v')
    } catch {
        return $null
    }
}

function Get-InstalledVersion {
    try {
        $jjitPath = Get-Command jjit -ErrorAction SilentlyContinue
        if ($jjitPath) {
            $versionOutput = & jjit --version 2>$null
            if ($versionOutput -match '(\d+\.\d+\.\d+)') {
                return $matches[1]
            }
        }
    } catch {}
    return $null
}

# Detect architecture
$arch = $env:PROCESSOR_ARCHITECTURE
if ($arch -eq "AMD64") {
    $target = "x86_64-pc-windows-msvc"
} elseif ($arch -eq "ARM64") {
    $target = "aarch64-pc-windows-msvc"
} else {
    Write-Error "Unsupported architecture: $arch"
    exit 1
}

Write-Info "Detected platform: Windows $arch"

# Check if already installed
$installedVersion = Get-InstalledVersion
$latestVersion = Get-LatestVersion

if ($installedVersion) {
    Write-Info "jjit is already installed (version: v$installedVersion)"
    
    if ($latestVersion) {
        Write-Info "Latest version: v$latestVersion"
        
        if ([version]$installedVersion -lt [version]$latestVersion) {
            Write-Info "Newer version available. Auto-updating..."
        } elseif ([version]$installedVersion -eq [version]$latestVersion) {
            Write-Info "Already at the latest version."
            $response = Read-Host "Reinstall anyway? [y/N]"
            if ($response -notin @('y', 'Y', 'yes', 'YES')) {
                Write-Info "Installation cancelled."
                exit 0
            }
        } else {
            Write-Warn "Installed version is newer than latest release."
            exit 0
        }
    }
}

# Create install directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# Download
$downloadUrl = "https://github.com/$Repo/releases/latest/download/jjit-$target.zip"
$tempDir = [System.IO.Path]::GetTempPath()
$zipFile = Join-Path $tempDir "jjit-$target.zip"

Write-Info "Downloading jjit from GitHub releases..."
Write-Info "  URL: $downloadUrl"

try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $zipFile -UseBasicParsing
} catch {
    Write-Error "Failed to download jjit: $_"
    exit 1
}

# Extract
Write-Info "Extracting..."
try {
    Expand-Archive -Path $zipFile -DestinationPath $tempDir -Force
} catch {
    Write-Error "Failed to extract archive: $_"
    exit 1
}

# Move binary to install directory
$extractedBinary = Join-Path $tempDir "jjit.exe"
if (Test-Path $extractedBinary) {
    Move-Item -Path $extractedBinary -Destination "$InstallDir\jjit.exe" -Force
} else {
    # Try to find binary in extracted folder
    $binary = Get-ChildItem -Path $tempDir -Filter "jjit.exe" -Recurse | Select-Object -First 1
    if ($binary) {
        Move-Item -Path $binary.FullName -Destination "$InstallDir\jjit.exe" -Force
    } else {
        Write-Error "jjit.exe not found in downloaded archive"
        exit 1
    }
}

# Clean up
Remove-Item -Path $zipFile -Force -ErrorAction SilentlyContinue

# Add to PATH if not already present
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$InstallDir*") {
    Write-Info "Adding $InstallDir to your PATH..."
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$InstallDir", "User")
    $env:Path = "$env:Path;$InstallDir"
}

# Verify installation
if (Test-Path "$InstallDir\jjit.exe") {
    Write-Success ""
    Write-Success "Successfully installed jjit!"
    & "$InstallDir\jjit.exe" --version
} else {
    Write-Error "Installation failed: jjit.exe not found in $InstallDir"
    exit 1
}

Write-Info ""
Write-Info "Next steps:"
Write-Info "  1. Set up your API key (globally):"
Write-Info "     jjit config set api_key sk-your-api-key-here --global"
Write-Info ""
Write-Info "  2. Or use environment variable:"
Write-Info "     `$env:DEEPSEEK_API_KEY = 'sk-your-api-key-here'"
Write-Info ""
Write-Info "  3. Try auto-commit:"
Write-Info "     jjit commit"
Write-Info ""
Write-Warn "Note: If 'jjit' command is not found, please restart your terminal or run:"
Write-Warn "  `$env:Path = [Environment]::GetEnvironmentVariable('Path', 'User')`"
