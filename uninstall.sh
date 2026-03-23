#!/usr/bin/env bash
# ────────────────────────────────────────────────────────────────────────────
# ani-nexus-tui uninstaller — Linux / macOS
# Usage: bash <(curl -sSf https://raw.githubusercontent.com/OsamuDazai666/ani-nexus-tui/main/uninstall.sh)
# ────────────────────────────────────────────────────────────────────────────
set -euo pipefail

INSTALL_DIR="${HOME}/.local/share/ani-nexus-tui"
BIN_DIR="${HOME}/.local/bin"
BINARY="${BIN_DIR}/ani-nexus"

# ── Colors ────────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; DIM='\033[2m'; RESET='\033[0m'

# ── Helpers ───────────────────────────────────────────────────────────────────
println()  { echo -e "$*"; }
header()   { println "\n  ${YELLOW}◆${RESET} ${BOLD}ANI-NEXUS-TUI UNINSTALLER${RESET}\n  ${DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}\n"; }
step()     { println "  ${CYAN}▶${RESET} ${BOLD}${1}${RESET}"; }
ok()       { println "    ${GREEN}✓${RESET} ${1}"; }
fail()     { println "    ${RED}✗${RESET} ${1}"; exit 1; }
info()     { println "    ${DIM}${1}${RESET}"; }
warn()     { println "    ${YELLOW}⚠${RESET}  ${1}"; }
sep()      { println "  ${DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"; }

ask() {
    printf "  ${CYAN}?${RESET} %s ${DIM}[Y/n]${RESET} " "$1"
    read -r ans </dev/tty
    [[ -z "$ans" || "$ans" =~ ^[Yy] ]]
}

# ── Start ─────────────────────────────────────────────────────────────────────
clear
header

if ! ask "Are you sure you want to completely remove ani-nexus-tui?"; then
    println "\n  Cancelled.\n"
    exit 0
fi

println ""

# ── Remove source files ───────────────────────────────────────────────────────
step "Removing source repository and build files"
if [[ -d "$INSTALL_DIR" ]]; then
    rm -rf "$INSTALL_DIR"
    ok "Deleted ${INSTALL_DIR}"
else
    ok "Source location already clean"
fi

# ── Remove executable ─────────────────────────────────────────────────────────
step "Removing executable binary"
if [[ -f "$BINARY" ]]; then
    rm -f "$BINARY"
    ok "Deleted ${BINARY}"
else
    ok "Binary location already clean"
fi

# ── Done ───────────────────────────────────────────────────────────────────────
println ""
sep
println "  ${YELLOW}◆${RESET} ${BOLD}Successfully uninstalled ani-nexus-tui${RESET}"
println ""
