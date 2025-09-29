# iro

**Fast, elegant wallpaper-based color scheme generator for Hyprland**

`iro` (色 - Japanese for "color") is a Rust-based dynamic theming tool that generates beautiful color schemes from your wallpapers and automatically applies them to your Hyprland desktop environment.

![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

## Features

- 🎨 **Intelligent Color Extraction** - Advanced algorithm extracts diverse, vibrant colors from wallpapers
- ⚡ **Blazing Fast** - Async thumbnail loading with parallel processing
- 🖼️ **Elegant GUI** - Minimalist lofi aesthetic with instant startup
- 🔄 **Automatic Theme Application** - Updates Hyprland, Waybar, and Kitty on the fly
- 🎯 **Smart Color Selection** - Avoids repetitive colors and ensures good contrast
- 🖥️ **Multi-Monitor Support** - Handles multiple displays seamlessly
- 🚀 **One-Command Setup** - Initialize everything with `--init`

## Installation

### Prerequisites

- Rust 1.70+
- Hyprland
- Waybar (optional)
- Kitty terminal (optional)
- `hyprpaper` for wallpaper management

### Building from Source

```bash
git clone https://github.com/0xdilo/iro.git
cd iro
cargo build --release
sudo cp target/release/iro /usr/local/bin/
```

## Quick Start

### 1. Initialize iro

```bash
iro --init
```

This will:
- Create `~/Pictures/wallpaper/` directory
- Set up `~/.config/iro/templates/`
- Install shell integration
- Configure Hyprland integration

### 2. Add Wallpapers

```bash
cp /path/to/your/wallpapers/* ~/Pictures/wallpaper/
```

### 3. Launch GUI

```bash
iro --gui
```

Select a wallpaper from the grid, and iro will:
- Extract colors from the image
- Generate theme files
- Reload Waybar and Hyprland
- Set the wallpaper across all monitors

## Usage

### GUI Mode (Recommended)

```bash
iro --gui
```

Browse your wallpapers in a beautiful grid layout with real-time thumbnails. Features:
- **Search** - Filter wallpapers by filename
- **Grid Adjustment** - Change column count (2-8)
- **Instant Preview** - Thumbnails load asynchronously
- **One-Click Apply** - Theme applies automatically

### CLI Mode

```bash
# Generate theme from specific wallpaper
iro /path/to/wallpaper.jpg

# Random wallpaper from ~/Pictures/wallpaper/
iro --random

# Use light theme
iro /path/to/wallpaper.jpg --theme light

# Specify monitors (comma-separated)
iro /path/to/wallpaper.jpg --monitors eDP-1,DP-3

# Generate and reload applications
iro /path/to/wallpaper.jpg --reload
```

## Configuration

### Template Customization

Templates are stored in `~/.config/iro/templates/`:

- `waybar_style.css` - Waybar color scheme
- `kitty.conf` - Kitty terminal colors
- `shell_colors.sh` - Shell color exports (for prompts, FZF, etc.)

Edit these templates to customize how colors are applied.

### Generated Files

iro creates/updates these files:

- `~/.config/iro/colors.sh` - Shell color variables
- `~/.config/hypr/hyprland.conf` - Hyprland color variables (appended)
- `~/.config/waybar/style.css` - Waybar theme
- `~/.config/kitty/kitty.conf` - Kitty colors (preserves existing config)

### Multi-Monitor Setup

iro automatically applies wallpapers to all monitors. For automatic wallpaper rotation on startup, add to your Hyprland config:

```conf
exec-once = iro --random
```

Or add it to your `~/.zshrc` / `~/.bashrc` for terminal startup theming.

## How It Works

1. **Color Extraction** - Resizes image and quantizes colors for performance
2. **Diversity Selection** - Ensures colors are distinct (50+ Euclidean distance)
3. **Anti-Green Filtering** - Prioritizes non-green accent colors
4. **Saturation Enhancement** - Boosts vibrancy by 40%
5. **Template Rendering** - Applies colors to config templates
6. **Application Reload** - Restarts Waybar and reloads Hyprland

## Color Scheme Format

Generated color schemes include:

- **Background** - Base window color
- **Foreground** - Primary text color
- **16 Terminal Colors** - Standard ANSI palette
- **Accent** - Primary highlight color (non-green)
- **Secondary** - Secondary accent (distinct from primary)
- **Surface** - Panel/widget background
- **Error** - Warning/error color

## Troubleshooting

### Waybar doesn't reload

Make sure waybar is running and restart it manually:

```bash
pkill waybar && waybar
```

### Colors look washed out

Adjust saturation factor in `src/color_extractor.rs`:

```rust
.map(|c| self.adjust_saturation(c, 1.6)) // Increase from 1.4
```

### GUI is slow with many wallpapers

iro uses async loading, but very large images may take time. Consider resizing wallpapers:

```bash
mogrify -resize 1920x1080 ~/Pictures/wallpaper/*.jpg
```

## Development

```bash
# Run in debug mode
cargo run -- --gui

# Run tests
cargo test

# Build release
cargo build --release

# Format code
cargo fmt
```

## Roadmap

- [ ] Support for additional compositors (Sway, i3)
- [ ] Color harmony algorithms (complementary, triadic)
- [ ] Palette export (JSON, YAML)
- [ ] Integration with Rofi/Wofi
- [ ] Live wallpaper preview
- [ ] Undo/redo color scheme history

## License

MIT License - See [LICENSE](LICENSE) for details

## Credits

Inspired by [pywal](https://github.com/dylanaraps/pywal) but built from scratch in Rust for maximum performance.

## Contributing

Contributions welcome! Please open an issue or PR.

---

**Made with 🦀 by [0xdilo](https://github.com/0xdilo)**