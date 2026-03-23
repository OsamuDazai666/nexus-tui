# ─────────────────────────────────────────────────────────────────────────────
# ani-nexus-tui uninstaller — Windows (PowerShell)
# Run with: powershell -ExecutionPolicy Bypass -File uninstall.ps1
# ─────────────────────────────────────────────────────────────────────────────
$ErrorActionPreference = "Stop"

trap {
    Write-Host ""
    Write-Host "    " -NoNewline
    Write-Host "✗ " -ForegroundColor Red -NoNewline
    Write-Host $_.Exception.Message -ForegroundColor Red
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 1
}

$INSTALL_DIR = Join-Path $env:APPDATA "ani-nexus-tui"
$BIN_DIR     = Join-Path $env:LOCALAPPDATA "Programs\ani-nexus-tui"

# ── Colours ───────────────────────────────────────────────────────────────────
function Write-Header {
    Clear-Host
    Write-Host ""
    Write-Host "  " -NoNewline
    Write-Host "◆ " -ForegroundColor Yellow -NoNewline
    Write-Host "ANI-NEXUS-TUI UNINSTALLER" -ForegroundColor White
    Write-Host "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor DarkGray
    Write-Host ""
}

function Write-Step($msg) {
    Write-Host "  " -NoNewline
    Write-Host "▶ " -ForegroundColor Cyan -NoNewline
    Write-Host $msg -ForegroundColor White
}

function Write-OK($msg) {
    Write-Host "    " -NoNewline
    Write-Host "✓ " -ForegroundColor Green -NoNewline
    Write-Host $msg -ForegroundColor Gray
}

function Write-Fail($msg) {
    Write-Host "    " -NoNewline
    Write-Host "✗ " -ForegroundColor Red -NoNewline
    Write-Host $msg -ForegroundColor Red
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 1
}

function Write-Info($msg)  { Write-Host "    $msg" -ForegroundColor DarkGray }

function Ask-YesNo($msg) {
    Write-Host "  " -NoNewline
    Write-Host "? " -ForegroundColor Cyan -NoNewline
    Write-Host "$msg " -ForegroundColor White -NoNewline
    Write-Host "[Y/n] " -ForegroundColor DarkGray -NoNewline
    $ans = Read-Host
    return ($ans -eq "" -or $ans -match "^[Yy]")
}

# ── Main ──────────────────────────────────────────────────────────────────────
Write-Header

if (-not (Ask-YesNo "Are you sure you want to completely remove ani-nexus-tui?")) {
    Write-Host ""; Write-Host "  Cancelled." -ForegroundColor DarkGray; Write-Host ""
    Read-Host "Press Enter to exit"
    exit 0
}

Write-Host ""

# ── Remove source files ───────────────────────────────────────────────────────
Write-Step "Removing source repository and build files"
if (Test-Path $INSTALL_DIR) {
    Remove-Item $INSTALL_DIR -Recurse -Force
    Write-OK "Deleted $INSTALL_DIR"
} else {
    Write-OK "Source location already clean"
}

# ── Remove executable ─────────────────────────────────────────────────────────
Write-Step "Removing executable binaries"
if (Test-Path $BIN_DIR) {
    Remove-Item $BIN_DIR -Recurse -Force
    Write-OK "Deleted $BIN_DIR"
} else {
    Write-OK "Binary location already clean"
}

# ── Remove from PATH ──────────────────────────────────────────────────────────
Write-Step "Cleaning up user PATH"
$userPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")

$paths = $userPath -split ";" | Where-Object { $_ -and $_ -ne $BIN_DIR -and $_ -ne "$BIN_DIR\" }
$newPath = $paths -join ";"

if ($userPath -ne $newPath) {
    [System.Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-OK "Removed $BIN_DIR from PATH"
} else {
    Write-OK "PATH already clean"
}

# ── Done ──────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor DarkGray
Write-Host "  " -NoNewline
Write-Host "◆ " -ForegroundColor Yellow -NoNewline
Write-Host "Successfully uninstalled ani-nexus-tui" -ForegroundColor White
Write-Host ""
Read-Host "Press Enter to exit"
