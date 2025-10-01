# Optional Setup Guide

After running `iro --init`, most setup is automatic. Here are optional configurations to enhance your experience:

## 1. Automatic Wallpaper on Startup

Add to your `~/.config/hypr/hyprland.conf`:

```conf
# Random wallpaper on startup (same on all monitors)
exec-once = iro --random

# OR: Different random wallpaper per monitor
exec-once = iro --random-each
```

This will apply a random wallpaper and color scheme every time Hyprland starts.

## 2. Keybindings

Add custom keybindings to `~/.config/hypr/hyprland.conf`:

```conf
# Switch to random wallpaper
bind = $mainMod SHIFT, W, exec, iro --random --reload

# Open wallpaper picker GUI
bind = $mainMod, W, exec, iro --gui
```

## 3. Shell Prompt Integration

If you use **Starship**, **Spaceship**, or **Powerlevel10k**, the colors are automatically exported to environment variables when you source `~/.config/iro/colors.sh` (already added to your `.bashrc`/`.zshrc` by `--init`).

Available color variables:
- `$BASE16_COLOR_00` through `$BASE16_COLOR_0F` (16 colors)
- `$SPACESHIP_CHAR_COLOR`, `$SPACESHIP_DIR_COLOR`, etc.

## 4. FZF Integration

FZF colors are automatically configured via `$FZF_DEFAULT_OPTS` when you source `~/.config/iro/colors.sh`.

No additional setup needed!

## 5. Custom Templates

You can customize how colors are applied by editing templates in `~/.config/iro/templates/`:

- `waybar.css` - Waybar color scheme
- `kitty.conf` - Kitty terminal colors
- `shell_colors.sh` - Shell color exports

**Note:** Templates use `{{ variable }}` syntax:
- `{{ background }}`, `{{ foreground }}`, `{{ accent }}`, `{{ secondary }}`
- `{{ colors.0 }}` through `{{ colors.15 }}` (16 terminal colors)
- `{{ red }}`, `{{ blue }}`, `{{ yellow }}`, etc. (named colors)

## 6. Different Wallpapers Per Monitor

If you have multiple monitors and want different wallpapers:

```bash
# Specify wallpapers in order (one per monitor)
iro ~/wallpaper1.jpg ~/wallpaper2.jpg

# Or use --random-each for different random wallpapers
iro --random-each

# Extract theme from second wallpaper instead of first
iro ~/wallpaper1.jpg ~/wallpaper2.jpg --primary 1
```

## 7. Light Theme

Use light theme instead of dark:

```bash
iro --random --theme light
```

## 8. Waybar Auto-Reload

By default, `iro --reload` (or `--gui`/`--random`) will automatically restart Waybar and reload Hyprland.

If you want manual control, omit the `--reload` flag:

```bash
iro /path/to/wallpaper.jpg  # No reload
```

Then reload manually:

```bash
pkill waybar && waybar &
hyprctl reload
```

---

**That's it!** Everything else is automatic. Enjoy your dynamic color schemes! 🎨
