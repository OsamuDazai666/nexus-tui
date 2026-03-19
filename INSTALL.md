# Installing nexus-tui

## One-line install

### Linux / macOS
```bash
curl -fsSL https://raw.githubusercontent.com/OsamuDazai666/nexus-tui/main/install.sh | bash
```

### Windows (PowerShell — run as Administrator)
```powershell
irm https://raw.githubusercontent.com/OsamuDazai666/nexus-tui/main/install.ps1 | iex
```

---

## What the installer does

### All platforms
1. Installs **Kitty terminal** (best image quality — cover art renders in full color)
2. Installs **mpv** (video player)
3. Installs **yt-dlp** (stream resolver for Movies/TV trailers)
4. Prompts for your **TMDB API key** and saves it to your shell profile
5. Downloads the pre-built `nexus` binary and adds it to your PATH

### Linux
Uses your existing package manager — `apt`, `pacman`, `dnf`, or `zypper`.
Falls back to building from source if no pre-built binary matches your platform.

### macOS
Installs Homebrew if not present, then uses `brew` for all dependencies.

### Windows
Installs Scoop if not present, then uses `winget`/`scoop` for dependencies.
Creates a Desktop shortcut that opens nexus directly inside Kitty.

---

## Manual install

Download the binary for your platform from the [latest release](https://github.com/YOU/nexus-tui/releases/latest):

| Platform       | File                          |
|----------------|-------------------------------|
| Linux x86_64   | `nexus-linux-x86_64`          |
| Linux ARM64    | `nexus-linux-aarch64`         |
| macOS x86_64   | `nexus-macos-x86_64`          |
| macOS ARM64    | `nexus-macos-aarch64`         |
| Windows x86_64 | `nexus-windows-x86_64.exe`    |

```bash
# Linux / macOS
chmod +x nexus-linux-x86_64
mv nexus-linux-x86_64 ~/.local/bin/nexus
```

---

## Dependencies

| Dependency | Required | Purpose |
|------------|----------|---------|
| mpv        | Yes      | Video playback |
| TMDB key   | For Movies/TV | Metadata + trailers |
| yt-dlp     | Recommended | Stream resolution for Movies/TV |
| Kitty      | Recommended | Full-color cover art |

Anime and Manga work without any API key.

---

## Building from source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Linux deps
sudo apt install build-essential pkg-config libssl-dev mpv yt-dlp

# Build
git clone https://github.com/YOU/nexus-tui
cd nexus-tui
cargo build --release

# Run
export TMDB_API_KEY="your_key"
./target/release/nexus
```
