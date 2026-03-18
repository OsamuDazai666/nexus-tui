# ◈ nexus-tui

**WARNING**: This is a work in progress and is not yet ready for production use.

A ~~blazing-fast~~ terminal UI for **Anime**, **Movies**, **TV** and **Manga** — 
website-quality browsing, zero browser required.

```
┌──────────────────────────────────────────────────────────────────┐
│  ◈ NEXUS  Anime[F1]│Movies[F2]│TV[F3]│Manga[F4]│History[F5]    │
├─────────────────┬──────────────────────────────────────────────  │
│ 🔍 cowboy bep.. │ [▓▓▓▓▓▓▓▓▓▓▓▓]  Cowboy Bebop                 │
│─────────────────│  ★ 8.9/10  ★★★★☆                             │
│ ▶ Cowboy Bebop  │  1998  ·  26 eps                              │
│   Trigun        │  ◉ Finished                                   │
│   Outlaw Star   │  [Action] [Sci-Fi] [Space] [Drama]            │
│   ...           │──────────────────────────────────────────────  │
│                 │  In the year 2071, humanity has colonized...   │
│                 │  ...                                           │
│─────────────────│──────────────────────────────────────────────  │
│ History         │  Related (8)                                   │
│  Ep 12/26 ████  │  ▶ Trigun  ★8.4  1998                        │
└─────────────────┴──────────────────────────────────────────────  │
│  [/] search  [j/k] navigate  [↵] select  [p] play  [q] quit     │
└──────────────────────────────────────────────────────────────────┘
```

## Setup

### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Install mpv
```bash
# Arch / Manjaro
sudo pacman -S mpv

# Ubuntu / Debian
sudo apt install mpv

# macOS
brew install mpv
```

### 3. Get a TMDB API key (free)
1. Go to https://www.themoviedb.org/settings/api
2. Create a free account and request an API key
3. Export it:
```bash
export TMDB_API_KEY="your_key_here"
# Or add to ~/.bashrc / ~/.zshrc
```

> AniList and MangaDex are completely free — no key needed.

### 4. Build & Run
```bash
git clone <repo>
cd nexus-tui
cargo run --release
# or after install:
nexus
```

## Keybindings

| Key | Action |
|-----|--------|
| `F1–F5` | Switch tab (Anime / Movies / TV / Manga / History) |
| `/` or `Tab` | Focus search bar |
| `Enter` | Execute search / select item |
| `j` / `k` or `↑↓` | Navigate list |
| `l` or `→` | Move focus to detail panel |
| `h` or `←` | Move focus back to results |
| `r` | Focus recommendations |
| `p` | Play in mpv |
| `d` | Delete from history (History tab) |
| `q` | Quit |
| `Ctrl+C` | Force quit |

## Image Rendering

nexus-tui auto-detects the best image protocol:

| Protocol | Terminal | Quality |
|----------|----------|---------|
| Kitty | Kitty, WezTerm | ★★★★★ |
| Sixel | xterm, foot, mlterm | ★★★★☆ |
| Half-blocks | All terminals | ★★★☆☆ |

## Content Sources

| Source | Content | Auth |
|--------|---------|------|
| AniList (GraphQL) | Anime | None |
| TMDB | Movies & TV | API key |
| MangaDex | Manga | None |

## Project Structure

```
src/
├── main.rs          # Entry point, terminal lifecycle
├── app.rs           # State machine, async message bus
├── api/
│   ├── mod.rs       # Unified ContentItem enum
│   ├── anilist.rs   # AniList GraphQL client
│   ├── tmdb.rs      # TMDB REST client
│   └── mangadex.rs  # MangaDex REST client
├── ui/
│   ├── mod.rs       # Layout composition, palette
│   ├── search.rs    # Search bar + results list
│   ├── detail.rs    # Meta, synopsis, recommendations
│   ├── image.rs     # Cover art renderer (halfblock/kitty/sixel)
│   └── history.rs   # History view + progress bars
├── db/
│   └── history.rs   # sled-backed persistent history
└── player.rs        # mpv launcher
```
