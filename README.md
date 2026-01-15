```
      /\_/\
     ( o.o )  iro ~
      > ^ <
```

# iro

a smol wallpaper-based color scheme generator :3

`iro` (色 - japanese for "color") extracts colors from wallpapers and applies them to hyprland, kitty, and more.

## features

- intelligent color extraction with hue-based mapping
- gui wallpaper selector
- multi-monitor support
- cute palette styles (kawaii, pastel, vivid, lofi, nord, warm, muted)
- auto-reload apps after theme change
- fast IPC-based wallpaper switching

## supported apps

- hyprland (+ hyprpaper)
- kitty
- waybar (optional)
- rofi (optional)
- quickshell (optional)

## install

```bash
git clone https://github.com/0xdilo/iro
cd iro
cargo build --release
cp target/release/iro ~/.cargo/bin/

# initialize iro (creates config, templates, wallpaper dir)
iro --init
```

## usage

```bash
# gui mode
iro --gui

# apply specific wallpaper
iro /path/to/wallpaper.jpg

# random wallpaper (same on all monitors)
iro --random

# random per monitor
iro --random-each

# light theme
iro --random --theme light
```

## config

`~/.config/iro/config.toml`:

```toml
[theme]
mode = "dark"  # dark, light
dark_background_style = "extracted"  # extracted, pure-dark, custom
light_background_style = "extracted"  # extracted, pure-light, custom

[palette]
style = "kawaii"  # kawaii, pastel, vivid, lofi, nord, warm, muted
diversity_threshold = 50.0
color_count = 16
```

### palette styles

| style | description |
|-------|-------------|
| kawaii | cute pink aesthetic with high saturation |
| pastel | soft dreamy pastels |
| vivid | bold vibrant colors |
| lofi | calm balanced aesthetic (default) |
| nord | cool nordic minimal |
| warm | cozy warm tones |
| muted | soft neutral palette |

## generated files

iro updates these files (only if the app is installed):

- `~/.config/hypr/hyprland.conf` - hyprland color variables
- `~/.config/hypr/hyprpaper.conf` - wallpaper config
- `~/.config/kitty/kitty.conf` - kitty colors
- `~/.config/waybar/style.css` - waybar theme
- `~/.config/rofi/config.rasi` - rofi colors
- `~/.config/quickshell/Theme.qml` - quickshell theme
- `~/.config/iro/colors.sh` - shell color exports

## hyprland setup

add to your `hyprland.conf`:

```conf
# start hyprpaper
exec-once = hyprpaper

# optional: random wallpaper on startup
exec-once = iro --random-each
```

## templates

custom templates are stored in `~/.config/iro/templates/`. edit them to customize the output format for each app.

available variables:
- `{{ background }}`, `{{ foreground }}`, `{{ accent }}`, `{{ secondary }}`, `{{ surface }}`, `{{ error }}`
- `{{ red }}`, `{{ yellow }}`, `{{ green }}`, `{{ cyan }}`, `{{ blue }}`, `{{ magenta }}`
- `{{ colors.0 }}` through `{{ colors.15 }}` for terminal colors

## license

do whatever u want with it lol
