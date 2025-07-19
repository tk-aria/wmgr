# wmgr PowerShell Installation Script
# This script downloads and installs wmgr for Windows systems

param(
    [string]$Version = $null,
    [string]$InstallPath = $null,
    [switch]$Force = $false,
    [switch]$Debug = $false
)

# Configuration
$DefaultInstallPath = "$env:LOCALAPPDATA\Programs\wmgr"
$RepoOwner = "tk-aria"
$RepoName = "wmgr"
$BinaryName = "wmgr.exe"

# Enable debug output if requested
if ($Debug) {
    $DebugPreference = 'Continue'
}

# Function to get latest release version
function Get-LatestVersion {
    try {
        Write-Host "Fetching latest version information..." -ForegroundColor Blue
        $apiUrl = "https://api.github.com/repos/$RepoOwner/$RepoName/releases/latest"
        $response = Invoke-RestMethod -Uri $apiUrl -ErrorAction Stop
        return $response.tag_name
    } catch {
        Write-Error "Failed to fetch latest version: $($_.Exception.Message)"
        exit 1
    }
}

# Function to detect architecture
function Get-Architecture {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64" { return "x86_64" }
        "ARM64" { return "aarch64" }
        default {
            Write-Error "Unsupported architecture: $arch"
            exit 1
        }
    }
}

# Function to get download URL
function Get-DownloadUrl {
    param(
        [string]$Version,
        [string]$Architecture
    )
    
    $archiveName = "wmgr-$Version-windows-$Architecture.tar.gz"
    return "https://github.com/$RepoOwner/$RepoName/releases/download/$Version/$archiveName"
}

# Function to create directory if it doesn't exist
function Ensure-Directory {
    param([string]$Path)
    
    if (-not (Test-Path $Path)) {
        Write-Debug "Creating directory: $Path"
        New-Item -ItemType Directory -Path $Path -Force | Out-Null
    }
}

# Function to download and extract wmgr
function Install-Wmgr {
    param(
        [string]$Version,
        [string]$InstallPath
    )
    
    $architecture = Get-Architecture
    $downloadUrl = Get-DownloadUrl -Version $Version -Architecture $architecture
    
    Write-Host "Installing wmgr $Version for Windows ($architecture)..." -ForegroundColor Green
    Write-Host "Download URL: $downloadUrl" -ForegroundColor Gray
    
    # Create install directory
    Ensure-Directory -Path $InstallPath
    
    # Create temporary directory
    $tempDir = Join-Path $env:TEMP "wmgr-install-$(Get-Random)"
    Ensure-Directory -Path $tempDir
    
    try {
        # Download archive
        $archivePath = Join-Path $tempDir "wmgr.tar.gz"
        Write-Host "Downloading wmgr archive..." -ForegroundColor Blue
        
        Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath -ErrorAction Stop
        
        # Extract archive using tar (available in Windows 10+)
        Write-Host "Extracting wmgr archive..." -ForegroundColor Blue
        
        if (Get-Command tar -ErrorAction SilentlyContinue) {
            & tar -xzf $archivePath -C $tempDir
        } else {
            Write-Error "tar command not found. Windows 10 (1803) or later is required."
            exit 1
        }
        
        # Move binary to install directory
        $sourceBinary = Join-Path $tempDir $BinaryName
        $targetBinary = Join-Path $InstallPath $BinaryName
        
        if (-not (Test-Path $sourceBinary)) {
            Write-Error "wmgr binary not found in extracted archive"
            exit 1
        }
        
        Write-Host "Installing wmgr to $targetBinary" -ForegroundColor Blue
        
        # Remove existing binary if it exists and force is specified
        if ((Test-Path $targetBinary) -and -not $Force) {
            Write-Error "wmgr is already installed at $targetBinary. Use -Force to overwrite."
            exit 1
        }
        
        Copy-Item $sourceBinary $targetBinary -Force
        
        # Verify installation
        if (Test-Path $targetBinary) {
            Write-Host ""
            Write-Host "✅ wmgr $Version has been successfully installed!" -ForegroundColor Green
            Write-Host ""
            Write-Host "The wmgr binary is installed at: $targetBinary" -ForegroundColor Gray
            Write-Host ""
            
            # Check if install directory is in PATH
            $pathEntries = $env:PATH -split ';'
            if ($InstallPath -notin $pathEntries) {
                Write-Host "⚠️  Warning: $InstallPath is not in your PATH." -ForegroundColor Yellow
                Write-Host "   To use wmgr from anywhere, add the following directory to your PATH:" -ForegroundColor Yellow
                Write-Host "   $InstallPath" -ForegroundColor White
                Write-Host ""
                Write-Host "   Or run wmgr using the full path:" -ForegroundColor Yellow
                Write-Host "   $targetBinary" -ForegroundColor White
            } else {
                Write-Host "To get started, run:" -ForegroundColor Gray
                Write-Host "  wmgr --help" -ForegroundColor White
            }
            
            Write-Host ""
            Write-Host "For more information, visit: https://github.com/$RepoOwner/$RepoName" -ForegroundColor Gray
        } else {
            Write-Error "Installation failed: wmgr binary not found at $targetBinary"
            exit 1
        }
        
    } catch {
        Write-Error "Installation failed: $($_.Exception.Message)"
        exit 1
    } finally {
        # Clean up temporary directory
        if (Test-Path $tempDir) {
            Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

# Main execution
try {
    Write-Host "wmgr PowerShell Installer" -ForegroundColor Cyan
    Write-Host "=========================" -ForegroundColor Cyan
    Write-Host ""
    
    # Determine version
    if (-not $Version) {
        $Version = Get-LatestVersion
        Write-Host "Latest version: $Version" -ForegroundColor Gray
    } else {
        Write-Host "Using specified version: $Version" -ForegroundColor Gray
    }
    
    # Determine install path
    if (-not $InstallPath) {
        $InstallPath = $DefaultInstallPath
    }
    Write-Host "Install path: $InstallPath" -ForegroundColor Gray
    Write-Host ""
    
    # Install wmgr
    Install-Wmgr -Version $Version -InstallPath $InstallPath
    
} catch {
    Write-Error "Unexpected error: $($_.Exception.Message)"
    exit 1
}

# Usage examples:
# .\install.ps1                              # Install latest version
# .\install.ps1 -Version "v1.0.0"           # Install specific version
# .\install.ps1 -InstallPath "C:\Tools"     # Custom install path
# .\install.ps1 -Force                      # Force overwrite existing installation
# .\install.ps1 -Debug                      # Enable debug output