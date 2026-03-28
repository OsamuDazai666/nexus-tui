param(
    [switch]$FromSource,
    [switch]$Offline,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$PassThruArgs = @()
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$repo = "OsamuDazai666/ani-nexus-tui"
$repoGitUrl = "https://github.com/$repo.git"
$installerUrl = "https://github.com/$repo/releases/latest/download/ani-nexus-tui-installer.ps1"
$zipAsset = "ani-nexus-tui-x86_64-pc-windows-msvc.zip"
$zipUrl = "https://github.com/$repo/releases/latest/download/$zipAsset"
$zipShaUrl = "$zipUrl.sha256"
$tempInstaller = Join-Path $env:TEMP "ani-nexus-tui-installer.ps1"
$tempZip = Join-Path $env:TEMP $zipAsset
$tempSha = Join-Path $env:TEMP ($zipAsset + ".sha256")
$extractDir = Join-Path $env:TEMP "ani-nexus-tui-extract"
$installDir = Join-Path $HOME ".cargo\bin"
$installedBinary = Join-Path $installDir "ani-nexus.exe"
$scriptRoot = if ($PSScriptRoot) { $PSScriptRoot } else { Split-Path -Parent $MyInvocation.MyCommand.Definition }
$downloadRetries = 3

function Set-TlsProtocol {
    try {
        $protocol = 0
        foreach ($name in @("Tls12", "Tls11", "Tls")) {
            if ([Enum]::GetNames([Net.SecurityProtocolType]) -contains $name) {
                $protocol = $protocol -bor [int][Net.SecurityProtocolType]::$name
            }
        }
        if ($protocol -ne 0) {
            [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]$protocol
        }
    } catch {}
}

function Write-Banner {
    Write-Host ""
    Write-Host "  ANI-NEXUS-TUI INSTALLER" -ForegroundColor White
    Write-Host "  release installer -> archive -> source fallback" -ForegroundColor DarkGray
    Write-Host ""
}

function Write-Step([string]$msg) {
    Write-Host "  " -NoNewline
    Write-Host "> " -ForegroundColor Cyan -NoNewline
    Write-Host $msg -ForegroundColor White
}

function Write-OK([string]$msg) {
    Write-Host "  " -NoNewline
    Write-Host "+ " -ForegroundColor Green -NoNewline
    Write-Host $msg -ForegroundColor Gray
}

function Write-Warn([string]$msg) {
    Write-Host "  " -NoNewline
    Write-Host "! " -ForegroundColor Yellow -NoNewline
    Write-Host $msg -ForegroundColor Yellow
}

function Ask-YesNo([string]$question) {
    if (-not [Environment]::UserInteractive) {
        return $false
    }
    Write-Host "  ? $question [Y/n]: " -NoNewline -ForegroundColor Cyan
    $answer = Read-Host
    return ([string]::IsNullOrWhiteSpace($answer) -or $answer -match '^[Yy]$')
}

function Require-Command([string]$name) {
    if (-not (Get-Command $name -ErrorAction SilentlyContinue)) {
        throw "Missing required command: $name"
    }
}

function Ensure-Path {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($null -eq $userPath) { $userPath = "" }
    $parts = @($userPath -split ';' | Where-Object { $_ -ne "" })
    $partsLower = @($parts | ForEach-Object { $_.ToLowerInvariant() })
    if ($partsLower -notcontains $installDir.ToLowerInvariant()) {
        if (Ask-YesNo "Add $installDir to your user PATH?") {
            if ($userPath) {
                $newPath = $userPath + ";" + $installDir
            } else {
                $newPath = $installDir
            }
            [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
            Write-OK "Added $installDir to user PATH"
        } else {
            $manualPath = if ($userPath) { $userPath + ";" + $installDir } else { $installDir }
            Write-Warn "PATH was not updated automatically"
            Write-Host "  Run this manually if needed:" -ForegroundColor DarkGray
            Write-Host "  [Environment]::SetEnvironmentVariable('Path', `"$manualPath`", 'User')" -ForegroundColor DarkGray
        }
    }
}

function Invoke-Download([string]$url, [string]$outFile, [int]$retries = 3) {
    for ($attempt = 1; $attempt -le $retries; $attempt++) {
        try {
            if (Get-Command Invoke-WebRequest -ErrorAction SilentlyContinue) {
                if ($PSVersionTable -and $PSVersionTable.PSVersion -and $PSVersionTable.PSVersion.Major -le 5) {
                    Invoke-WebRequest -Uri $url -OutFile $outFile -UseBasicParsing
                } else {
                    Invoke-WebRequest -Uri $url -OutFile $outFile
                }
            } else {
                $wc = New-Object Net.WebClient
                try {
                    $wc.DownloadFile($url, $outFile)
                } finally {
                    $wc.Dispose()
                }
            }
            return $true
        } catch {
            if ($attempt -eq $retries) {
                return $false
            }
            Start-Sleep -Seconds 2
        }
    }
    return $false
}

function Get-FileSha256([string]$path) {
    if (Get-Command Get-FileHash -ErrorAction SilentlyContinue) {
        return (Get-FileHash -Algorithm SHA256 -Path $path).Hash.ToLowerInvariant()
    }

    if (Get-Command certutil.exe -ErrorAction SilentlyContinue) {
        $hashLine = certutil.exe -hashfile $path SHA256 | Select-Object -Skip 1 -First 1
        return (($hashLine -replace '\s+', '').ToLowerInvariant())
    }

    $stream = [System.IO.File]::OpenRead($path)
    try {
        $sha = [System.Security.Cryptography.SHA256]::Create()
        try {
            $bytes = $sha.ComputeHash($stream)
            return ([BitConverter]::ToString($bytes) -replace '-', '').ToLowerInvariant()
        } finally {
            $sha.Dispose()
        }
    } finally {
        $stream.Dispose()
    }
}

function Verify-ChecksumIfAvailable([string]$filePath, [string]$checksumUrl, [string]$checksumPath) {
    if (-not (Invoke-Download -url $checksumUrl -outFile $checksumPath -retries $downloadRetries)) {
        Write-Warn "Checksum asset missing ($checksumUrl), skipping integrity verification"
        return
    }

    $line = (Get-Content -Path $checksumPath -ErrorAction Stop | Select-Object -First 1)
    $expected = ""
    if ($line -match '([A-Fa-f0-9]{64})') {
        $expected = $Matches[1].ToLowerInvariant()
    }
    if (-not $expected) {
        throw "Invalid checksum file format for $checksumUrl"
    }

    $actual = Get-FileSha256 -path $filePath
    if ($actual -ne $expected) {
        throw "Checksum verification failed for $filePath"
    }
    Write-OK "Checksum verified for $(Split-Path $filePath -Leaf)"
}

function Extract-ZipCompat([string]$zipPath, [string]$destination) {
    if (Test-Path $destination) {
        Remove-Item $destination -Recurse -Force -ErrorAction SilentlyContinue
    }
    New-Item -ItemType Directory -Path $destination -Force | Out-Null

    if (Get-Command Expand-Archive -ErrorAction SilentlyContinue) {
        Expand-Archive -Path $zipPath -DestinationPath $destination -Force
        return
    }

    try {
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        [System.IO.Compression.ZipFile]::ExtractToDirectory($zipPath, $destination)
        return
    } catch {}

    $shell = New-Object -ComObject Shell.Application
    $zipNs = $shell.NameSpace($zipPath)
    $destNs = $shell.NameSpace($destination)
    if (-not $zipNs -or -not $destNs) {
        throw "Unable to extract zip archive in this environment."
    }
    $destNs.CopyHere($zipNs.Items(), 16)
    Start-Sleep -Seconds 1
}

function Install-FromArchive {
    Write-Step "Downloading release archive ($zipAsset)"
    if (-not (Invoke-Download -url $zipUrl -outFile $tempZip -retries $downloadRetries)) {
        Write-Warn "Release archive download failed"
        return $false
    }

    Verify-ChecksumIfAvailable -filePath $tempZip -checksumUrl $zipShaUrl -checksumPath $tempSha
    Extract-ZipCompat -zipPath $tempZip -destination $extractDir

    $binary = Get-ChildItem -Path $extractDir -Recurse -File | Where-Object { $_.Name -eq "ani-nexus.exe" } | Select-Object -First 1
    if (-not $binary) {
        Write-Warn "Could not locate ani-nexus.exe in archive"
        return $false
    }

    Ensure-Path
    Copy-Item $binary.FullName $installedBinary -Force
    Write-OK "Installed ani-nexus to $installedBinary"
    return $true
}

function Resolve-LocalSourcePath {
    if (Test-Path (Join-Path $scriptRoot "Cargo.toml")) {
        return $scriptRoot
    }
    if (Test-Path (Join-Path (Get-Location) "Cargo.toml")) {
        return (Get-Location).Path
    }
    return $null
}

function Install-FromSource {
    Require-Command cargo
    $localPath = Resolve-LocalSourcePath

    Write-Step "Installing from source with cargo"
    if ($localPath) {
        Write-Host "    Using local source at $localPath" -ForegroundColor DarkGray
        cargo install --path $localPath --locked --force | Out-Host
    } else {
        if ($Offline) {
            throw "Offline mode requires running from a local ani-nexus-tui checkout."
        }
        Require-Command git
        Write-Host "    Using git source at $repoGitUrl" -ForegroundColor DarkGray
        cargo install ani-nexus-tui --git $repoGitUrl --locked --force | Out-Host
    }

    Ensure-Path
    Write-OK "Installed ani-nexus via cargo"
}

function Invoke-ReleaseInstaller {
    Write-Step "Fetching release installer script"
    if (-not (Invoke-Download -url $installerUrl -outFile $tempInstaller -retries $downloadRetries)) {
        Write-Warn "Release installer asset not found"
        return $false
    }

    & $tempInstaller @PassThruArgs
    Write-OK "Completed via release installer"
    return $true
}

try {
    Set-TlsProtocol
    Write-Banner

    if ($FromSource -or $Offline) {
        Install-FromSource
    } else {
        if (-not (Invoke-ReleaseInstaller)) {
            if (-not (Install-FromArchive)) {
                Write-Warn "Archive install failed, trying source install"
                Install-FromSource
            }
        }
    }

    Write-Host ""
    Write-OK "Ready. Run: ani-nexus --version"
    Write-Host "  Open a new terminal if command is not found." -ForegroundColor DarkGray
}
finally {
    Remove-Item $tempInstaller -Force -ErrorAction SilentlyContinue
    Remove-Item $tempZip -Force -ErrorAction SilentlyContinue
    Remove-Item $tempSha -Force -ErrorAction SilentlyContinue
    Remove-Item $extractDir -Recurse -Force -ErrorAction SilentlyContinue
}
