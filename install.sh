#!/usr/bin/env bash
set -e

BOLD="\033[1m"
CYAN="\033[36m"
GREEN="\033[32m"
YELLOW="\033[33m"
RED="\033[31m"
RESET="\033[0m"

echo -e "${CYAN}${BOLD}"
echo "  ◈ nexus-tui installer"
echo -e "${RESET}"

# Check Rust
if ! command -v cargo &>/dev/null; then
    echo -e "${YELLOW}Rust not found. Installing...${RESET}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

echo -e "${GREEN}✓ Rust $(rustc --version)${RESET}"

# Check ani-cli (best stream source — mirrors exactly what we use)
if ! command -v ani-cli &>/dev/null; then
    echo -e "${YELLOW}⚠  ani-cli not found — recommended for reliable anime streaming${RESET}"
    echo "   Install on Ubuntu/Debian:"
    echo "     sudo apt install ani-cli"
    echo "   Or from source:"
    echo "     sudo curl -fsSL https://raw.githubusercontent.com/pystardust/ani-cli/master/ani-cli -o /usr/local/bin/ani-cli"
    echo "     sudo chmod +x /usr/local/bin/ani-cli"
else
    echo -e "${GREEN}✓ ani-cli found — anime streaming will use it directly${RESET}"
fi

# Check yt-dlp (fallback stream resolver + YouTube for movies)
if ! command -v yt-dlp &>/dev/null; then
    echo -e "${YELLOW}⚠  yt-dlp not found (needed for YouTube trailers + stream fallback)${RESET}"
    echo "   Install: sudo apt install yt-dlp"
else
    echo -e "${GREEN}✓ yt-dlp $(yt-dlp --version)${RESET}"
fi

# Check mpv
if ! command -v mpv &>/dev/null; then
    echo -e "${YELLOW}⚠  mpv not found. Install it:${RESET}"
    echo "   Arch:   sudo pacman -S mpv"
    echo "   Ubuntu: sudo apt install mpv"
    echo "   macOS:  brew install mpv"
    echo ""
fi

# TMDB key prompt
if [ -z "$TMDB_API_KEY" ]; then
    echo -e "${YELLOW}TMDB_API_KEY not set.${RESET}"
    echo "  Get a free key at: https://www.themoviedb.org/settings/api"
    read -rp "  Enter your TMDB API key (or press Enter to skip for now): " KEY
    if [ -n "$KEY" ]; then
        export TMDB_API_KEY="$KEY"
        SHELL_RC=""
        if [ -f "$HOME/.zshrc" ]; then SHELL_RC="$HOME/.zshrc"
        elif [ -f "$HOME/.bashrc" ]; then SHELL_RC="$HOME/.bashrc"
        fi
        if [ -n "$SHELL_RC" ]; then
            echo "export TMDB_API_KEY=\"$KEY\"" >> "$SHELL_RC"
            echo -e "${GREEN}✓ Saved to $SHELL_RC${RESET}"
        fi
    fi
fi

# Build
echo -e "\n${CYAN}Building nexus-tui (release)...${RESET}"
cargo build --release

# Install binary
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
cp target/release/nexus "$INSTALL_DIR/nexus"
echo -e "${GREEN}✓ Installed to $INSTALL_DIR/nexus${RESET}"

# PATH check
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}⚠  Add this to your shell profile:${RESET}"
    echo "   export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

echo -e "\n${GREEN}${BOLD}Done! Run:  nexus${RESET}\n"
