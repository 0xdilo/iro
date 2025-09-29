# iro Configuration Guide

iro now supports extensive configuration through `~/.config/iro/config.toml`. The config file will be automatically created on first run with default values.

## Configuration File Location

`~/.config/iro/config.toml`

## Theme Configuration

### `[theme]`

#### `mode`
- **Type:** String
- **Options:** `"dark"`, `"light"`, `"auto"`
- **Default:** `"dark"`
- **Description:** Sets the overall theme mode. Use `"auto"` for future automatic switching based on time of day.

#### `dark_background_style`
- **Type:** String
- **Options:**
  - `"extracted"` - Generate a dark background from wallpaper colors
  - `"pure-dark"` - Use a solid dark background (#1e1e2e)
  - `"custom"` - Use a custom hex color
- **Default:** `"extracted"`
- **Description:** Controls how the background is generated in dark mode.

#### `dark_background_custom`
- **Type:** String (optional)
- **Format:** Hex color (e.g., `"#1a1b26"`)
- **Default:** `None`
- **Description:** Custom background color for dark mode. Only used when `dark_background_style = "custom"`.

#### `light_background_style`
- **Type:** String
- **Options:**
  - `"extracted"` - Generate a light background from wallpaper colors
  - `"pure-light"` - Use a solid light background (#eff1f5)
  - `"custom"` - Use a custom hex color
- **Default:** `"pure-light"`
- **Description:** Controls how the background is generated in light mode.

#### `light_background_custom`
- **Type:** String (optional)
- **Format:** Hex color (e.g., `"#faf4ed"`)
- **Default:** `None`
- **Description:** Custom background color for light mode. Only used when `light_background_style = "custom"`.

## Palette Configuration

### `[palette]`

#### `style`
- **Type:** String
- **Options:**
  - `"vibrant"` - Balanced, vivid colors (default)
  - `"pastel"` - Soft, muted colors with low saturation
  - `"neon"` - Ultra-vibrant, electric colors
  - `"muted"` - Subtle, professional tones
  - `"catppuccin"` - Soothing pastel palette
  - `"nord"` - Arctic, bluish palette
  - `"dracula"` - Dark, vibrant purple tones
  - `"gruvbox"` - Warm, retro earth tones
  - `"tokyo-night"` - Clean, modern blue-purple
  - `"rose-pine"` - Elegant, muted rose tones
- **Default:** `"vibrant"`
- **Description:** Preset style that determines the overall aesthetic and color treatment of the palette.

#### `diversity_threshold`
- **Type:** Float
- **Range:** 0.0 - 200.0
- **Default:** `50.0`
- **Description:** Controls how different colors need to be to be included in the palette. Higher values result in more diverse color palettes, lower values allow similar colors.

#### `dark_saturation`
- **Type:** Float
- **Range:** 0.0 - 3.0
- **Default:** `1.5`
- **Description:** Saturation multiplier for colors in dark mode. Values > 1.0 make colors more vibrant, < 1.0 make them more muted.

#### `light_saturation`
- **Type:** Float
- **Range:** 0.0 - 3.0
- **Default:** `1.2`
- **Description:** Saturation multiplier for colors in light mode.

#### `light_brightness`
- **Type:** Float
- **Range:** 0.0 - 2.0
- **Default:** `0.8`
- **Description:** Brightness multiplier for colors in light mode. Values < 1.0 darken colors for better readability on light backgrounds.

#### `color_count`
- **Type:** Integer
- **Range:** 8 - 32
- **Default:** `16`
- **Description:** Number of distinct colors to extract from the wallpaper. Standard terminal themes use 16 colors.

## Example Configurations

### Neon Cyberpunk Theme
```toml
[theme]
mode = "dark"
dark_background_style = "extracted"
light_background_style = "pure-light"

[palette]
style = "neon"
diversity_threshold = 60.0
dark_saturation = 2.2
light_saturation = 1.8
light_brightness = 0.85
color_count = 16
```

### Catppuccin Cozy Theme
```toml
[theme]
mode = "dark"
dark_background_style = "extracted"
light_background_style = "extracted"

[palette]
style = "catppuccin"
diversity_threshold = 45.0
dark_saturation = 1.1
light_saturation = 0.9
light_brightness = 0.92
color_count = 16
```

### Gruvbox Retro Theme
```toml
[theme]
mode = "dark"
dark_background_style = "extracted"
light_background_style = "extracted"

[palette]
style = "gruvbox"
diversity_threshold = 50.0
dark_saturation = 1.2
light_saturation = 1.0
light_brightness = 0.82
color_count = 16
```

### Muted Dark Theme with Pure Background
```toml
[theme]
mode = "dark"
dark_background_style = "pure-dark"
light_background_style = "pure-light"

[palette]
diversity_threshold = 45.0
dark_saturation = 1.2
light_saturation = 1.0
light_brightness = 0.8
color_count = 16
```

### Custom Background Colors
```toml
[theme]
mode = "dark"
dark_background_style = "custom"
dark_background_custom = "#1a1b26"
light_background_style = "custom"
light_background_custom = "#faf4ed"

[palette]
diversity_threshold = 50.0
dark_saturation = 1.5
light_saturation = 1.2
light_brightness = 0.8
color_count = 16
```

### High Contrast Theme
```toml
[theme]
mode = "dark"
dark_background_style = "pure-dark"
light_background_style = "pure-light"

[palette]
diversity_threshold = 80.0
dark_saturation = 2.0
light_saturation = 1.5
light_brightness = 0.7
color_count = 20
```

## Tips

1. **Extracted backgrounds** work best with wallpapers that have a dominant color theme
2. **Higher diversity threshold** (60-80) creates more distinct colors but may miss subtle tones
3. **Lower diversity threshold** (30-40) captures more color variations but may include similar shades
4. **Dark saturation** around 1.5-1.8 provides vibrant colors without being overwhelming
5. **Light brightness** around 0.7-0.8 ensures good contrast on light backgrounds
6. Experiment with `color_count` between 16-20 for wallpapers with many distinct colors

## Command Line Override

The config file provides defaults, but you can still override the theme mode via command line:

```bash
# Force dark mode regardless of config
iro --theme dark ~/wallpaper.jpg

# Force light mode regardless of config
iro --theme light ~/wallpaper.jpg
```

## Reloading Configuration

Configuration changes take effect immediately on the next run of `iro`. No restart required.