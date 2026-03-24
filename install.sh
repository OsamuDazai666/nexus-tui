#!/usr/bin/env bash
# ────────────────────────────────────────────────────────────────────────────
# ani-nexus-tui installer — Linux / macOS (Arch & Fedora compatible)
# ────────────────────────────────────────────────────────────────────────────

set -uo pipefail

REPO_URL="https://github.com/OsamuDazai666/ani-nexus-tui.git"
INSTALL_DIR="${HOME}/.local/share/ani-nexus-tui"
BIN_DIR="${HOME}/.local/bin"
BINARY="${INSTALL_DIR}/target/release/ani-nexus"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; DIM='\033[2m'; RESET='\033[0m'

println() { echo -e "$*"; }
header() { clear 2>/dev/null || true; println "\n  ${YELLOW}◆${RESET} ${BOLD}ANI-NEXUS-TUI INSTALLER${RESET}\n  ${DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}\n"; }
step() { println "  ${CYAN}▶${RESET} ${BOLD}${1}${RESET}"; }
ok() { println "    ${GREEN}✓${RESET} ${1}"; }
fail() { println "    ${RED}✗${RESET} ${1}"; exit 1; }
info() { println "    ${DIM}${1}${RESET}"; }
warn() { println "    ${YELLOW}⚠${RESET}  ${1}"; }
sep() { println "  ${DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"; }

ask() {
    printf "  ${CYAN}?${RESET} %s ${DIM}[Y/n]${RESET} " "$1"
    read -r ans </dev/tty
    [[ -z "$ans" || "$ans" =~ ^[Yy] ]]
}

spin() {
    local pid=$1 msg="$2"
    local frames=('⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏')
    local i=0
    while ! kill -0 "$pid" 2>/dev/null && [[ $i -lt 50 ]]; do sleep 0.05; ((i++)); done
    i=0
    while kill -0 "$pid" 2>/dev/null; do
        printf "\r    ${CYAN}%s${RESET}  %s" "${frames[$((i % 10))]}" "$msg"
        sleep 0.08; ((i++)) || true
    done
    printf "\r%-60s\r" ""
}

quiet_install() {
    local cmd="$1" msg="$2" log="/tmp/ani-install-$$.log"
    
    (eval "$cmd" >"$log" 2>&1) &
    spin $! "$msg"
    wait $!
    local exit_code=$?
    
    if [[ $exit_code -ne 0 ]]; then
        println ""
        warn "Installation failed (exit $exit_code)"
        [[ -f "$log" ]] && tail -10 "$log" | sed 's/^/      /'
        rm -f "$log"
        return 1
    fi
    
    rm -f "$log"
    return 0
}

# ── FIXED: Proper package detection for each distro ───────────────────────────
get_install_info() {
    local type="$1"
    
    # Ubuntu/Debian (apt)
    if command -v apt &>/dev/null; then
        [[ "$type" == "openssl" ]] && echo "apt|libssl-dev pkg-config"
        [[ "$type" == "mpv" ]] && echo "apt|mpv"
        [[ "$type" == "git" ]] && echo "apt|git"
        [[ "$type" == "curl" ]] && echo "apt|curl"
    
    # Debian fallback (apt-get)
    elif command -v apt-get &>/dev/null; then
        [[ "$type" == "openssl" ]] && echo "apt-get|libssl-dev pkg-config"
        [[ "$type" == "mpv" ]] && echo "apt-get|mpv"
        [[ "$type" == "git" ]] && echo "apt-get|git"
        [[ "$type" == "curl" ]] && echo "apt-get|curl"
    
    # Fedora (dnf)
    elif command -v dnf &>/dev/null; then
        [[ "$type" == "openssl" ]] && echo "dnf|openssl-devel pkgconf"
        [[ "$type" == "mpv" ]] && echo "dnf|mpv"
        [[ "$type" == "git" ]] && echo "dnf|git"
        [[ "$type" == "curl" ]] && echo "dnf|curl"
    
    # RHEL/CentOS legacy (yum)
    elif command -v yum &>/dev/null; then
        [[ "$type" == "openssl" ]] && echo "yum|openssl-devel pkgconfig"
        [[ "$type" == "mpv" ]] && echo "yum|mpv"
        [[ "$type" == "git" ]] && echo "yum|git"
        [[ "$type" == "curl" ]] && echo "yum|curl"
    
    # Arch (pacman) - FIXED: no --quiet flag
    elif command -v pacman &>/dev/null; then
        [[ "$type" == "openssl" ]] && echo "pacman|openssl pkgconf"
        [[ "$type" == "mpv" ]] && echo "pacman|mpv"
        [[ "$type" == "git" ]] && echo "pacman|git"
        [[ "$type" == "curl" ]] && echo "pacman|curl"
    
    # macOS (brew)
    elif [[ "$(uname)" == "Darwin" ]] && command -v brew &>/dev/null; then
        [[ "$type" == "openssl" ]] && echo "brew|openssl pkg-config"
        [[ "$type" == "mpv" ]] && echo "brew|mpv"
        [[ "$type" == "git" ]] && echo "brew|git"
        [[ "$type" == "curl" ]] && echo "brew|curl"
    fi
}

ensure_openssl() {
    if pkg-config --exists openssl 2>/dev/null; then
        ok "openssl-dev ($(pkg-config --modversion openssl))"
        return 0
    fi

    warn "OpenSSL development libraries not found"
    println ""
    
    local info install_cmd pkg_mgr pkgs
    info=$(get_install_info "openssl")
    [[ -z "$info" ]] && fail "Cannot detect package manager"
    
    pkg_mgr="${info%%|*}"
    pkgs="${info##*|}"
    
    info "Will install: $pkgs (via $pkg_mgr)"
    println ""

    if ! ask "Install OpenSSL development libraries?"; then
        fail "OpenSSL development libraries are required"
    fi

    println ""
    step "Installing OpenSSL dev libraries"
    
    # FIXED: Proper quiet flags for each package manager
    case "$pkg_mgr" in
        apt)
            install_cmd="sudo apt update -qq && sudo apt install -y -qq $pkgs"
            ;;
        apt-get)
            install_cmd="sudo apt-get update -qq && sudo apt-get install -y -qq $pkgs"
            ;;
        dnf)
            install_cmd="sudo dnf install -y -q $pkgs"
            ;;
        yum)
            install_cmd="sudo yum install -y -q $pkgs"
            ;;
        pacman)
            # FIXED: pacman has no --quiet for -S, redirect stderr
            install_cmd="sudo pacman -S --noconfirm $pkgs 2>/dev/null"
            ;;
        brew)
            install_cmd="brew install $pkgs 2>/dev/null"
            ;;
    esac
    
    if quiet_install "$install_cmd" "Installing OpenSSL dev libraries..."; then
        # Re-verify
        if pkg-config --exists openssl 2>/dev/null; then
            ok "OpenSSL dev libraries installed"
        else
            # For Arch, might need to re-source or update pkgconfig path
            [[ "$pkg_mgr" == "pacman" ]] && export PKG_CONFIG_PATH=/usr/lib/pkgconfig:$PKG_CONFIG_PATH
            if pkg-config --exists openssl 2>/dev/null; then
                ok "OpenSSL dev libraries installed"
            else
                fail "OpenSSL still not found after install - try: export PKG_CONFIG_PATH=/usr/lib/pkgconfig"
            fi
        fi
    else
        fail "Failed to install OpenSSL dev libraries"
    fi
}

ensure_rust() {
    if command -v rustc &>/dev/null && command -v cargo &>/dev/null; then
        ok "rust $(rustc --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)"
        return 0
    fi

    warn "Rust not found"
    if ! ask "Install Rust via rustup?"; then
        fail "Rust is required"
    fi

    println ""
    step "Installing Rust via rustup"
    
    (
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh &&
        sh /tmp/rustup-init.sh -y --quiet 2>/dev/null
    ) &
    spin $! "Installing Rust toolchain..."
    wait $! || fail "Rust installation failed"
    
    [[ -f "${HOME}/.cargo/env" ]] && source "${HOME}/.cargo/env"
    
    if command -v cargo &>/dev/null; then
        ok "Rust installed ($(rustc --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1))"
    else
        fail "cargo not in PATH after install - run: source ~/.cargo/env"
    fi
}

ensure_mpv() {
    if command -v mpv &>/dev/null; then
        local mpv_ver
        mpv_ver=$(mpv --version 2>/dev/null | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)
        ok "mpv ${mpv_ver}"
        return 0
    fi

    warn "mpv not found (required to play anime)"
    println ""
    
    local info install_cmd pkg_mgr pkgs
    info=$(get_install_info "mpv")
    [[ -z "$info" ]] && { warn "Cannot auto-install mpv - install manually"; return 0; }
    
    pkg_mgr="${info%%|*}"
    pkgs="${info##*|}"
    
    info "Will install: $pkgs (via $pkg_mgr)"
    println ""

    if ! ask "Install mpv media player?"; then
        warn "Skipping mpv install (you won't be able to play videos)"
        return 0
    fi

    println ""
    step "Installing mpv"
    
    # FIXED: Same quiet flags pattern
    case "$pkg_mgr" in
        apt)      install_cmd="sudo apt install -y -qq $pkgs" ;;
        apt-get)  install_cmd="sudo apt-get install -y -qq $pkgs" ;;
        dnf)      install_cmd="sudo dnf install -y -q $pkgs" ;;
        yum)      install_cmd="sudo yum install -y -q $pkgs" ;;
        pacman)   install_cmd="sudo pacman -S --noconfirm $pkgs 2>/dev/null" ;;
        brew)     install_cmd="brew install $pkgs 2>/dev/null" ;;
    esac
    
    if quiet_install "$install_cmd" "Installing mpv media player..."; then
        ok "mpv installed"
    else
        warn "Failed to install mpv - you may need to install manually"
    fi
}

install_pkg_generic() {
    local pkg_type="$1" pkg_name="$2"
    local info install_cmd pkg_mgr pkgs
    
    info=$(get_install_info "$pkg_type")
    [[ -z "$info" ]] && fail "Cannot detect package manager for $pkg_name install"
    
    pkg_mgr="${info%%|*}"
    pkgs="${info##*|}"
    
    step "Installing $pkg_name"
    
    case "$pkg_mgr" in
        apt)      install_cmd="sudo apt install -y -qq $pkgs" ;;
        apt-get)  install_cmd="sudo apt-get install -y -qq $pkgs" ;;
        dnf)      install_cmd="sudo dnf install -y -q $pkgs" ;;
        yum)      install_cmd="sudo yum install -y -q $pkgs" ;;
        pacman)   install_cmd="sudo pacman -S --noconfirm $pkgs 2>/dev/null" ;;
        brew)     install_cmd="brew install $pkgs 2>/dev/null" ;;
    esac
    
    quiet_install "$install_cmd" "Installing $pkg_name..." && ok "$pkg_name installed" || fail "Failed to install $pkg_name"
}

header

# ── Check OpenSSL ──────────────────────────────────────────────────────────────
step "Checking OpenSSL development libraries"
ensure_openssl
println ""

# ── Check Rust ─────────────────────────────────────────────────────────────────
step "Checking Rust toolchain"
ensure_rust
println ""

# ── Check other deps ───────────────────────────────────────────────────────────
step "Checking other dependencies"

# Git
if ! command -v git &>/dev/null; then
    warn "git not found"
    info "Will install git"
    println ""
    if ask "Install git?"; then
        install_pkg_generic "git" "git"
    else
        fail "git is required"
    fi
else
    ok "git"
fi

# Curl
if ! command -v curl &>/dev/null; then
    warn "curl not found"
    info "Will install curl"
    println ""
    if ask "Install curl?"; then
        install_pkg_generic "curl" "curl"
    else
        fail "curl is required"
    fi
else
    ok "curl"
fi

# MPV
ensure_mpv
println ""

# ── Repository handling ─────────────────────────────────────────────────────────
if [[ -d "${INSTALL_DIR}/.git" ]]; then
    println "  ${YELLOW}Existing install detected${RESET}"
    println ""
    step "Checking for updates"

    cd "$INSTALL_DIR" || fail "Cannot cd to install dir"
    git fetch origin --quiet 2>/dev/null || true

    LOCAL=$(git rev-parse HEAD 2>/dev/null || echo "none")
    REMOTE=$(git rev-parse origin/main 2>/dev/null || echo "unknown")

    if [[ "$LOCAL" == "$REMOTE" ]]; then
        ok "Already up to date"
        if [[ -f "${BIN_DIR}/ani-nexus" ]]; then
            println ""
            info "Run ani-nexus to launch"
            println ""
            exit 0
        fi
        warn "Binary not found — will rebuild"
        println ""
    else
        COMMITS=$(git log --oneline "${LOCAL}..${REMOTE}" 2>/dev/null | wc -l | tr -d ' ')
        info "${COMMITS} new commit(s) available"
        println ""

        if ! ask "Update ani-nexus-tui?"; then
            println "\n  Skipped.\n"
            exit 0
        fi

        println ""
        step "Pulling latest"
        git pull origin main --quiet || fail "git pull failed"
        ok "Pulled $(git rev-parse --short HEAD)"
        println ""
    fi
else
    info "Install location: ${INSTALL_DIR}"
    println ""

    if ! ask "Install ani-nexus-tui?"; then
        println "\n  Cancelled.\n"
        exit 0
    fi

    println ""

    step "Cloning repository"
    [[ -d "$INSTALL_DIR" ]] && rm -rf "$INSTALL_DIR"
    mkdir -p "$(dirname "$INSTALL_DIR")" || fail "Cannot create install dir parent"
    
    (git clone --quiet "$REPO_URL" "$INSTALL_DIR" 2>&1) &
    spin $! "Cloning ani-nexus-tui..."
    wait $! || fail "git clone failed"
    
    ok "Cloned to ${INSTALL_DIR}"
    println ""
fi

# ── Build ──────────────────────────────────────────────────────────────────────
step "Building ani-nexus-tui"
info "First build takes 2–5 minutes"
println ""

cd "$INSTALL_DIR" || fail "Cannot cd to build dir"

if ! command -v cargo &>/dev/null; then
    [[ -f "${HOME}/.cargo/env" ]] && source "${HOME}/.cargo/env"
fi
command -v cargo &>/dev/null || fail "cargo not found - run: source ~/.cargo/env"

START=$(date +%s)
BUILD_LOG="/tmp/ani-nexus-build.log"

println "    ${DIM}Compiling... (this may take a while)${RESET}"

if ! CARGO_INCREMENTAL=0 cargo build --release >"$BUILD_LOG" 2>&1; then
    BUILD_EXIT=$?
    println ""
    println "  ${RED}Build failed with exit code ${BUILD_EXIT}:${RESET}"
    println ""
    tail -40 "$BUILD_LOG" | sed 's/^/    /'
    println ""
    fail "Build failed - see errors above"
fi

END=$(date +%s)
ok "Built in $((END - START))s"
println ""

# ── Install binary ─────────────────────────────────────────────────────────────
step "Installing binary"
mkdir -p "$BIN_DIR" || fail "Cannot create bin dir"
[[ -f "$BINARY" ]] || fail "Binary not found at $BINARY"
cp "$BINARY" "${BIN_DIR}/ani-nexus" || fail "Cannot copy binary"
chmod +x "${BIN_DIR}/ani-nexus" || fail "Cannot chmod binary"
ok "Installed to ${BIN_DIR}/ani-nexus"

if [[ ":${PATH}:" != *":${BIN_DIR}:"* ]]; then
    println ""
    warn "${BIN_DIR} is not in your PATH"
    info "Add this to ~/.bashrc or ~/.zshrc:"
    println ""
    println "    export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

println ""
sep
println "  ${YELLOW}◆${RESET} ${BOLD}Done!${RESET}  Run ${CYAN}${BOLD}ani-nexus${RESET} to launch"
println ""