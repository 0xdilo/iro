# iro

Fast wallpaper-based color scheme generator for Hyprland

`iro` (色 - Japanese for "color") generates vibrant color schemes from wallpapers and automatically applies them to Hyprland, Waybar, Kitty, and Rofi.

## Features

- Intelligent color extraction with hue-based terminal color mapping
- Fast async thumbnail loading
- GUI wallpaper selector
- Automatic theme application
- Multi-monitor support
- Configurable palette styles (lofi, nord, warm, muted)

## Installation

### Prerequisites

- Rust 1.70+
- Hyprland
- `hyprpaper` for wallpaper management
- Waybar (optional)
- Kitty terminal (optional)
- Rofi launcher (optional)

### Building

```bash
git clone <your-repo-url>
cd iro
cargo build --release
cp target/release/iro ~/.cargo/bin/
```

## Quick Start

### Initialize iro

```bash
iro --init
```

This creates:
- `~/Pictures/wallpaper/` directory
- `~/.config/iro/` config and templates
- Shell integration in `.bashrc`/`.zshrc`

### Add wallpapers

```bash
cp /path/to/wallpapers/* ~/Pictures/wallpaper/
```

## Usage

### GUI Mode

```bash
iro --gui
```

Browse wallpapers in a grid, search by filename, and apply themes with one click.

### CLI Mode

```bash
# Generate theme from specific wallpaper
iro /path/to/wallpaper.jpg

# Random wallpaper (same on all monitors)
iro --random

# Random wallpaper per monitor
iro --random-each

# Light theme
iro /path/to/wallpaper.jpg --theme light

# Specify monitors
iro /path/to/wallpaper.jpg --monitors eDP-1,DP-3
```

## Configuration

Config file: `~/.config/iro/config.toml`

```toml
[theme]
mode = "dark"  # "dark", "light", or "auto"
dark_background_style = "extracted"  # "extracted", "pure-dark", or "custom"
light_background_style = "extracted"  # "extracted", "pure-light", or "custom"

[palette]
style = "lofi"  # "lofi", "nord", "warm", "muted"
diversity_threshold = 50.0
color_count = 16
```

### Palette Styles

- **lofi** - Calm balanced aesthetic
- **nord** - Cool nordic minimal
- **warm** - Cozy warm tones
- **muted** - Soft neutral palette

### Generated Files

- `~/.config/iro/colors.sh` - Shell color variables
- `~/.config/hypr/hyprland.conf` - Hyprland colors (section replaced)
- `~/.config/waybar/style.css` - Waybar theme
- `~/.config/kitty/kitty.conf` - Kitty colors (section replaced)
- `~/.config/rofi/config.rasi` - Rofi colors (section replaced)

Original configs are backed up with `.iro.bak` extension.

## How It Works

1. **Color Extraction** - Extracts dominant colors from resized image
2. **Hue Mapping** - Maps colors to proper terminal hues (red, yellow, green, cyan, blue, magenta)
3. **Synthetic Generation** - Generates missing colors in hue ranges
4. **Saturation Boost** - Enhances color vibrancy for better terminal visibility
5. **Theme Application** - Updates config files and reloads applications

## Development

```bash
# Run in debug mode
cargo run -- --gui

# Build release
cargo build --release
```
