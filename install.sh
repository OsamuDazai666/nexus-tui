#!/usr/bin/env bash
set -euo pipefail

REPO="OsamuDazai666/ani-nexus-tui"
INSTALLER_URL="https://github.com/${REPO}/releases/latest/download/ani-nexus-tui-installer.sh"
REPO_GIT_URL="https://github.com/${REPO}.git"
BINARY_NAME="ani-nexus"
PACKAGE_NAME="ani-nexus-tui"
ALIAS_NAME="ani-nexus-tui"
INSTALL_DIR="${CARGO_HOME:-$HOME/.cargo}/bin"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOWNLOAD_RETRIES=3
RETRY_DELAY=2

if [[ -t 1 ]]; then
    C_RESET=$'\033[0m'
    C_BOLD=$'\033[1m'
    C_DIM=$'\033[2m'
    C_CYAN=$'\033[36m'
    C_GREEN=$'\033[32m'
    C_YELLOW=$'\033[33m'
    C_RED=$'\033[31m'
else
    C_RESET=""
    C_BOLD=""
    C_DIM=""
    C_CYAN=""
    C_GREEN=""
    C_YELLOW=""
    C_RED=""
fi

step() { printf "%b\n" "  ${C_CYAN}>${C_RESET} ${C_BOLD}$*${C_RESET}"; }
ok() { printf "%b\n" "  ${C_GREEN}+${C_RESET} $*"; }
warn() { printf "%b\n" "  ${C_YELLOW}!${C_RESET} $*"; }
fail() { printf "%b\n" "  ${C_RED}x${C_RESET} $*" >&2; exit 1; }
info() { printf "%b\n" "  ${C_DIM}$*${C_RESET}"; }

banner() {
    printf "%b\n" ""
    printf "%b\n" "  ${C_BOLD}ANI-NEXUS-TUI INSTALLER${C_RESET}"
    printf "%b\n" "  ${C_DIM}release installer -> archive -> source fallback${C_RESET}"
    printf "%b\n" ""
}

command_exists() { command -v "$1" >/dev/null 2>&1; }

require_cmds() {
    local missing=0 cmd
    for cmd in "$@"; do
        if ! command_exists "$cmd"; then
            warn "Missing required command: $cmd"
            missing=1
        fi
    done
    [[ $missing -eq 0 ]] || fail "Install prerequisites missing"
}

download_with_retry() {
    local url="$1" out="$2" attempt
    for ((attempt=1; attempt<=DOWNLOAD_RETRIES; attempt++)); do
        if curl --proto '=https' --tlsv1.2 -fLsS "$url" -o "$out"; then
            return 0
        fi
        if [[ "$attempt" -lt "$DOWNLOAD_RETRIES" ]]; then
            info "Retrying download (${attempt}/${DOWNLOAD_RETRIES})..."
            sleep "$RETRY_DELAY"
        fi
    done
    return 1
}

detect_sha_tool() {
    if command_exists sha256sum; then
        echo "sha256sum"
    elif command_exists shasum; then
        echo "shasum"
    else
        echo ""
    fi
}

verify_checksum_if_available() {
    local asset_path="$1" checksum_url="$2" checksum_file hash_tool actual expected
    checksum_file="$(mktemp)"
    if ! download_with_retry "$checksum_url" "$checksum_file"; then
        warn "Checksum asset missing (${checksum_url##*/}), skipping integrity verification"
        rm -f "$checksum_file"
        return 0
    fi

    hash_tool="$(detect_sha_tool)"
    if [[ -z "$hash_tool" ]]; then
        warn "No SHA256 tool available (sha256sum/shasum), skipping integrity verification"
        rm -f "$checksum_file"
        return 0
    fi

    expected="$(awk '{print $1}' "$checksum_file" | tr -d '\r\n' || true)"
    rm -f "$checksum_file"
    [[ -n "$expected" ]] || fail "Invalid checksum file format"

    if [[ "$hash_tool" == "sha256sum" ]]; then
        actual="$(sha256sum "$asset_path" | awk '{print $1}')"
    else
        actual="$(shasum -a 256 "$asset_path" | awk '{print $1}')"
    fi

    [[ "$actual" == "$expected" ]] || fail "Checksum verification failed for ${asset_path##*/}"
    ok "Checksum verified for ${asset_path##*/}"
}

ensure_alias() {
    ln -sf "${INSTALL_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${ALIAS_NAME}" 2>/dev/null || true
}

path_contains_install_dir() {
    local entry
    IFS=':' read -r -a path_entries <<< "${PATH:-}"
    for entry in "${path_entries[@]}"; do
        [[ "$entry" == "$INSTALL_DIR" ]] && return 0
    done
    return 1
}

prompt_yes_no() {
    local prompt="$1" response
    if [[ ! -t 0 || ! -t 1 ]]; then
        return 1
    fi
    printf "%b" "  ${C_CYAN}?${C_RESET} ${prompt} [Y/n]: "
    read -r response
    [[ -z "$response" || "$response" =~ ^[Yy]$ ]]
}

add_path_to_shell_profile() {
    local rc_file line
    case "${SHELL##*/}" in
        zsh) rc_file="${HOME}/.zshrc" ;;
        bash) rc_file="${HOME}/.bashrc" ;;
        *) rc_file="${HOME}/.profile" ;;
    esac

    line="export PATH=\"${INSTALL_DIR}:\$PATH\""
    touch "$rc_file"
    if ! grep -Fq "$line" "$rc_file" 2>/dev/null; then
        printf "\n%s\n" "$line" >> "$rc_file"
    fi
    ok "Added ${INSTALL_DIR} to PATH in ${rc_file}"
}

ensure_path() {
    if path_contains_install_dir; then
        return 0
    fi

    if prompt_yes_no "Add ${INSTALL_DIR} to PATH in your shell profile?"; then
        add_path_to_shell_profile
    else
        warn "PATH was not updated automatically"
        info "Add manually: export PATH=\"${INSTALL_DIR}:\$PATH\""
    fi
}

preflight_archive_install() {
    require_cmds curl tar find install mktemp uname
}

preflight_source_install() {
    require_cmds cargo
    if [[ "${OFFLINE_MODE}" -eq 0 ]] && [[ -z "${SOURCE_PATH:-}" ]]; then
        require_cmds git
    fi
}

resolve_source_path() {
    if [[ -f "${SCRIPT_DIR}/Cargo.toml" ]]; then
        SOURCE_PATH="${SCRIPT_DIR}"
    elif [[ -f "${PWD}/Cargo.toml" ]]; then
        SOURCE_PATH="${PWD}"
    else
        SOURCE_PATH=""
    fi
}

install_from_archive() {
    local os arch asset archive_url checksum_url tmp_dir archive_file binary_path
    preflight_archive_install

    os="$(uname -s)"
    arch="$(uname -m)"

    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
        *) fail "Unsupported architecture: $arch" ;;
    esac

    case "$os" in
        Linux) asset="ani-nexus-tui-${arch}-unknown-linux-gnu.tar.xz" ;;
        Darwin) asset="ani-nexus-tui-${arch}-apple-darwin.tar.xz" ;;
        *) fail "Unsupported OS: $os" ;;
    esac

    archive_url="https://github.com/${REPO}/releases/latest/download/${asset}"
    checksum_url="${archive_url}.sha256"
    tmp_dir="$(mktemp -d)"
    archive_file="${tmp_dir}/${asset}"

    step "Downloading release archive (${asset})"
    download_with_retry "$archive_url" "$archive_file" || { rm -rf "$tmp_dir"; return 1; }
    verify_checksum_if_available "$archive_file" "$checksum_url"

    tar -xf "$archive_file" -C "$tmp_dir" || { rm -rf "$tmp_dir"; return 1; }
    binary_path="$(find "$tmp_dir" -type f -name "$BINARY_NAME" -perm -u+x | head -n 1)"
    [[ -n "$binary_path" ]] || { rm -rf "$tmp_dir"; return 1; }

    mkdir -p "$INSTALL_DIR"
    install -m 755 "$binary_path" "${INSTALL_DIR}/${BINARY_NAME}" || { rm -rf "$tmp_dir"; return 1; }
    ensure_alias
    ensure_path
    rm -rf "$tmp_dir"
    ok "Installed ${BINARY_NAME} to ${INSTALL_DIR}"
}

install_from_source() {
    resolve_source_path
    preflight_source_install

    step "Installing from source with cargo"
    if [[ -n "$SOURCE_PATH" ]]; then
        info "Using local source at ${SOURCE_PATH}"
        cargo install --path "$SOURCE_PATH" --locked --force
    else
        if [[ "${OFFLINE_MODE}" -eq 1 ]]; then
            fail "Offline mode requires local checkout (run script from repo)"
        fi
        info "Using git source at ${REPO_GIT_URL}"
        cargo install "$PACKAGE_NAME" --git "$REPO_GIT_URL" --locked --force
    fi

    mkdir -p "$INSTALL_DIR"
    ensure_alias
    ensure_path
    ok "Installed ${BINARY_NAME} via cargo"
}

try_release_installer() {
    local tmp_file
    tmp_file="$(mktemp)"
    step "Fetching release installer script"
    if download_with_retry "$INSTALLER_URL" "$tmp_file"; then
        sh "$tmp_file" "${PASSTHROUGH_ARGS[@]}"
        rm -f "$tmp_file"
        ok "Completed via release installer"
        return 0
    fi
    rm -f "$tmp_file"
    warn "Release installer asset not found"
    return 1
}

usage() {
    cat <<EOF
Usage: install.sh [options] [-- <args passed to release installer>]

Options:
  --from-source, --build-from-source  Force source installation
  --offline                           Disable network fallback to git/source remote
  --help                              Show this message
EOF
}

OFFLINE_MODE=0
FORCE_SOURCE=0
PASSTHROUGH_ARGS=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        --offline) OFFLINE_MODE=1 ;;
        --from-source|--build-from-source) FORCE_SOURCE=1 ;;
        --help|-h) usage; exit 0 ;;
        --) shift; PASSTHROUGH_ARGS+=("$@"); break ;;
        *) PASSTHROUGH_ARGS+=("$1") ;;
    esac
    shift
done

banner

if [[ "$FORCE_SOURCE" -eq 1 || "$OFFLINE_MODE" -eq 1 ]]; then
    install_from_source
else
    if ! try_release_installer; then
        if ! install_from_archive; then
            warn "Archive install failed, trying source install"
            install_from_source
        fi
    fi
fi

printf "%b\n" ""
ok "Ready. Run: ${BINARY_NAME} --version"
