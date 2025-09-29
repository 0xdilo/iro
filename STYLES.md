# Palette Styles Guide

iro includes **4 carefully curated styles** focused on calm, nordic, lofi aesthetics. All styles use very low saturation (0.35-0.42) for that relaxed, elegant feel.

## The 4 Styles

### lofi (Default)
**Description:** Calm balanced aesthetic
**Saturation:** 0.42 (dark) / 0.37 (light)
**Contrast:** 0.7 (soft)
**Warmth:** Slightly warm (+5%)

**Perfect for:** General use, balanced between cool and warm. Calm and versatile.

```toml
[palette]
style = "lofi"
```

---

### nord
**Description:** Cool nordic minimal
**Saturation:** 0.35 (dark) / 0.3 (light)
**Contrast:** 0.65 (very soft)
**Warmth:** Cool (-12%)

**Perfect for:** True nordic aesthetic. Very cool blue tones. Minimalist and clean.

```toml
[palette]
style = "nord"
```

---

### warm
**Description:** Cozy warm tones
**Saturation:** 0.4 (dark) / 0.35 (light)
**Contrast:** 0.68 (soft)
**Warmth:** Warm (+15%)

**Perfect for:** Cozy evening coding sessions. Warm oranges and browns. Comfortable.

```toml
[palette]
style = "warm"
```

---

### muted
**Description:** Soft neutral palette
**Saturation:** 0.38 (dark) / 0.33 (light)
**Contrast:** 0.67 (very soft)
**Warmth:** Neutral (+2%)

**Perfect for:** Maximum subtlety. Almost grayscale with hint of color. Professional and understated.

```toml
[palette]
style = "muted"
```

---

## Comparison

| Style | Saturation | Warmth | Best For |
|-------|-----------|--------|----------|
| **lofi** | 0.42 | Slightly warm | Balanced, versatile |
| **nord** | 0.35 | Cool | Nordic, minimal, blue |
| **warm** | 0.40 | Very warm | Cozy, comfortable |
| **muted** | 0.38 | Neutral | Professional, subtle |

## Using Styles

### In GUI
1. Launch `iro --gui`
2. Click 🌙/☀ to toggle dark/light mode
3. Click the style button (single letter icon)
4. Choose from the 4 styles
5. Select wallpaper and apply

### In Config
Edit `~/.config/iro/config.toml`:
```toml
[palette]
style = "nord"  # or lofi, warm, muted
```

## Philosophy

All 4 styles share these principles:
- **Very low saturation** (0.35-0.42) - No vibrant colors
- **Soft contrast** (0.65-0.7) - Easy on the eyes
- **Subdued brightness** (0.82-0.85) - Calm, not harsh
- **Distinct character** - Each style has clear identity

This creates that perfect **calm, nordic, lofi, relaxed** aesthetic for unixporn setups.