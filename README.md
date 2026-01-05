```
      /\_/\
     ( o.o )  iro ~
      > ^ <
```

# iro

a smol wallpaper-based color scheme generator :3

`iro` (色 - japanese for "color") extracts colors from wallpapers and applies them to hyprland, waybar, kitty, rofi, and quickshell.

## features

- intelligent color extraction with hue-based mapping
- gui wallpaper selector
- multi-monitor support
- palette styles (lofi, nord, warm, muted)
- auto-reload apps after theme change

## supported apps

- hyprland
- waybar
- kitty
- rofi
- quickshell ([quickshell-dotfiles](https://github.com/0xdilo/quickshell-dotfiles))

## install

```bash
git clone https://github.com/0xdilo/iro
cd iro
cargo build --release
cp target/release/iro ~/.cargo/bin/
```

## usage

```bash
# gui mode
iro --gui

# apply specific wallpaper
iro /path/to/wallpaper.jpg

# random wallpaper
iro --random

# random per monitor
iro --random-each
```

## config

`~/.config/iro/config.toml`:

```toml
[theme]
mode = "dark"  # dark, light, auto

[palette]
style = "lofi"  # lofi, nord, warm, muted
```

## generated files

- `~/.config/iro/colors.sh` - shell variables
- `~/.config/hypr/hyprland.conf` - hyprland colors
- `~/.config/waybar/style.css` - waybar theme
- `~/.config/kitty/kitty.conf` - kitty colors
- `~/.config/rofi/config.rasi` - rofi colors
- `~/Git/quick/Theme.qml` - quickshell theme

## license

do whatever u want with it lol
