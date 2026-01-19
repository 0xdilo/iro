use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IroConfig {
    pub theme: ThemeConfig,
    pub palette: PaletteConfig,
    /// Directory containing wallpaper images
    #[serde(default = "default_wallpaper_dir")]
    pub wallpaper_dir: String,
}

fn default_wallpaper_dir() -> String {
    dirs::home_dir()
        .map(|h| {
            h.join("Pictures")
                .join("Wallpaper")
                .to_string_lossy()
                .to_string()
        })
        .unwrap_or_else(|| "~/Pictures/Wallpaper".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Theme mode: "light" or "dark" or "auto"
    pub mode: String,

    /// Background color style for dark mode
    /// "extracted" - use darkened color from wallpaper
    /// "pure-dark" - use pure dark background (#1e1e2e)
    /// "custom" - use custom hex color
    pub dark_background_style: String,

    /// Custom background color (used when dark_background_style is "custom")
    pub dark_background_custom: Option<String>,

    /// Background color style for light mode
    /// "extracted" - use lightened color from wallpaper
    /// "pure-light" - use pure light background (#eff1f5)
    /// "custom" - use custom hex color
    pub light_background_style: String,

    /// Custom background color (used when light_background_style is "custom")
    pub light_background_custom: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteConfig {
    /// Palette style preset
    /// Options: "vibrant", "pastel", "neon", "muted", "catppuccin", "nord", "dracula", "gruvbox", "tokyo-night", "rose-pine"
    pub style: String,

    /// Color diversity threshold (higher = more diverse colors required)
    pub diversity_threshold: f32,

    /// Saturation boost for dark mode (1.0 = no boost, >1.0 = more saturated)
    pub dark_saturation: f32,

    /// Saturation adjustment for light mode
    pub light_saturation: f32,

    /// Brightness adjustment for light mode
    pub light_brightness: f32,

    /// Number of colors to extract from image
    pub color_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorHarmony {
    Extracted,
    Analogous,
    Triadic,
    SplitComp,
    Complementary,
}

#[derive(Debug, Clone)]
pub struct PaletteStyle {
    pub description: &'static str,
    pub dark_saturation: f32,
    pub light_saturation: f32,
    pub dark_brightness: f32,
    pub light_brightness: f32,
    pub contrast: f32,
    pub warmth_shift: f32,
    pub hue_boosts: &'static [(f32, f32, f32)],
    pub target_hues: Option<&'static [f32]>,
    pub bg_tint_strength: f32,
    pub color_harmony: ColorHarmony,
}

impl PaletteStyle {
    pub fn from_name(name: &str) -> Self {
        match name {
            "kawaii" => Self {
                description: "Cute pink aesthetic",
                dark_saturation: 0.55,
                light_saturation: 0.50,
                dark_brightness: 0.88,
                light_brightness: 0.92,
                contrast: 0.75,
                warmth_shift: 0.25,
                hue_boosts: &[(320.0, 60.0, 0.18), (270.0, 40.0, 0.12)],
                target_hues: Some(&[330.0, 280.0, 200.0]),
                bg_tint_strength: 0.14,
                color_harmony: ColorHarmony::Analogous,
            },
            "pastel" => Self {
                description: "Soft dreamy pastels",
                dark_saturation: 0.45,
                light_saturation: 0.40,
                dark_brightness: 0.90,
                light_brightness: 0.95,
                contrast: 0.60,
                warmth_shift: 0.10,
                hue_boosts: &[],
                target_hues: None,
                bg_tint_strength: 0.12,
                color_harmony: ColorHarmony::Analogous,
            },
            "vivid" => Self {
                description: "Bold vibrant colors",
                dark_saturation: 0.65,
                light_saturation: 0.55,
                dark_brightness: 0.85,
                light_brightness: 0.88,
                contrast: 0.85,
                warmth_shift: 0.0,
                hue_boosts: &[],
                target_hues: None,
                bg_tint_strength: 0.08,
                color_harmony: ColorHarmony::Triadic,
            },
            "nord" => Self {
                description: "Cool nordic minimal",
                dark_saturation: 0.35,
                light_saturation: 0.30,
                dark_brightness: 0.82,
                light_brightness: 0.88,
                contrast: 0.65,
                warmth_shift: -0.12,
                hue_boosts: &[(200.0, 50.0, 0.10), (170.0, 40.0, 0.08)],
                target_hues: Some(&[200.0, 180.0, 220.0]),
                bg_tint_strength: 0.10,
                color_harmony: ColorHarmony::Analogous,
            },
            "warm" => Self {
                description: "Cozy warm tones",
                dark_saturation: 0.45,
                light_saturation: 0.40,
                dark_brightness: 0.85,
                light_brightness: 0.88,
                contrast: 0.70,
                warmth_shift: 0.18,
                hue_boosts: &[(30.0, 40.0, 0.12), (15.0, 30.0, 0.10)],
                target_hues: Some(&[30.0, 45.0, 15.0]),
                bg_tint_strength: 0.15,
                color_harmony: ColorHarmony::Analogous,
            },
            "muted" => Self {
                description: "Soft neutral palette",
                dark_saturation: 0.38,
                light_saturation: 0.33,
                dark_brightness: 0.84,
                light_brightness: 0.88,
                contrast: 0.67,
                warmth_shift: 0.02,
                hue_boosts: &[],
                target_hues: None,
                bg_tint_strength: 0.10,
                color_harmony: ColorHarmony::Extracted,
            },
            "catppuccin" => Self {
                description: "Creamy pastels, mocha vibes",
                dark_saturation: 0.52,
                light_saturation: 0.48,
                dark_brightness: 0.88,
                light_brightness: 0.92,
                contrast: 0.72,
                warmth_shift: 0.08,
                hue_boosts: &[(15.0, 30.0, 0.10), (220.0, 40.0, 0.08)],
                target_hues: Some(&[350.0, 220.0, 170.0, 45.0]),
                bg_tint_strength: 0.15,
                color_harmony: ColorHarmony::Analogous,
            },
            "dracula" => Self {
                description: "Purple/pink gothic aesthetic",
                dark_saturation: 0.65,
                light_saturation: 0.55,
                dark_brightness: 0.86,
                light_brightness: 0.88,
                contrast: 0.78,
                warmth_shift: -0.05,
                hue_boosts: &[(300.0, 60.0, 0.20), (280.0, 40.0, 0.15)],
                target_hues: Some(&[300.0, 280.0, 330.0, 180.0]),
                bg_tint_strength: 0.12,
                color_harmony: ColorHarmony::SplitComp,
            },
            "gruvbox" => Self {
                description: "Retro warm oranges, earthy",
                dark_saturation: 0.55,
                light_saturation: 0.50,
                dark_brightness: 0.84,
                light_brightness: 0.86,
                contrast: 0.80,
                warmth_shift: 0.22,
                hue_boosts: &[(35.0, 30.0, 0.15), (100.0, 40.0, 0.10)],
                target_hues: Some(&[40.0, 100.0, 180.0, 0.0]),
                bg_tint_strength: 0.18,
                color_harmony: ColorHarmony::Complementary,
            },
            "tokyo-night" => Self {
                description: "Cool neon blues, city night",
                dark_saturation: 0.58,
                light_saturation: 0.50,
                dark_brightness: 0.86,
                light_brightness: 0.88,
                contrast: 0.75,
                warmth_shift: -0.15,
                hue_boosts: &[(185.0, 30.0, 0.18), (230.0, 40.0, 0.12)],
                target_hues: Some(&[185.0, 230.0, 280.0, 340.0]),
                bg_tint_strength: 0.10,
                color_harmony: ColorHarmony::Analogous,
            },
            "rose-pine" => Self {
                description: "Romantic muted rose tints",
                dark_saturation: 0.42,
                light_saturation: 0.38,
                dark_brightness: 0.88,
                light_brightness: 0.92,
                contrast: 0.68,
                warmth_shift: 0.12,
                hue_boosts: &[(340.0, 40.0, 0.15), (275.0, 30.0, 0.08)],
                target_hues: Some(&[340.0, 275.0, 45.0, 190.0]),
                bg_tint_strength: 0.14,
                color_harmony: ColorHarmony::Analogous,
            },
            "everforest" => Self {
                description: "Nature greens, forest calm",
                dark_saturation: 0.45,
                light_saturation: 0.42,
                dark_brightness: 0.86,
                light_brightness: 0.90,
                contrast: 0.70,
                warmth_shift: 0.02,
                hue_boosts: &[(120.0, 60.0, 0.20), (55.0, 30.0, 0.08)],
                target_hues: Some(&[120.0, 90.0, 55.0, 180.0]),
                bg_tint_strength: 0.12,
                color_harmony: ColorHarmony::Analogous,
            },
            "synthwave" => Self {
                description: "Neon retro 80s vibes",
                dark_saturation: 0.75,
                light_saturation: 0.65,
                dark_brightness: 0.88,
                light_brightness: 0.85,
                contrast: 0.90,
                warmth_shift: 0.0,
                hue_boosts: &[(325.0, 30.0, 0.25), (185.0, 20.0, 0.25)],
                target_hues: Some(&[325.0, 185.0, 280.0, 45.0]),
                bg_tint_strength: 0.08,
                color_harmony: ColorHarmony::Complementary,
            },
            _ => Self {
                description: "Calm balanced aesthetic",
                dark_saturation: 0.48,
                light_saturation: 0.42,
                dark_brightness: 0.86,
                light_brightness: 0.90,
                contrast: 0.72,
                warmth_shift: 0.08,
                hue_boosts: &[],
                target_hues: None,
                bg_tint_strength: 0.12,
                color_harmony: ColorHarmony::Extracted,
            },
        }
    }

    pub fn all_styles() -> Vec<&'static str> {
        vec![
            "lofi",
            "kawaii",
            "pastel",
            "vivid",
            "nord",
            "warm",
            "muted",
            "catppuccin",
            "dracula",
            "gruvbox",
            "tokyo-night",
            "rose-pine",
            "everforest",
            "synthwave",
        ]
    }
}

impl Default for IroConfig {
    fn default() -> Self {
        Self {
            theme: ThemeConfig {
                mode: "dark".to_string(),
                dark_background_style: "extracted".to_string(),
                dark_background_custom: None,
                light_background_style: "extracted".to_string(),
                light_background_custom: None,
            },
            palette: PaletteConfig {
                style: "lofi".to_string(),
                diversity_threshold: 50.0,
                dark_saturation: 0.42,
                light_saturation: 0.37,
                light_brightness: 0.88,
                color_count: 16,
            },
            wallpaper_dir: default_wallpaper_dir(),
        }
    }
}

impl IroConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            // Create default config
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let content =
            std::fs::read_to_string(&config_path).context("Failed to read config file")?;

        let config: IroConfig = toml::from_str(&content).context("Failed to parse config file")?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        std::fs::write(&config_path, content).context("Failed to write config file")?;

        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Failed to get config directory")?;
        Ok(config_dir.join("iro").join("config.toml"))
    }

    pub fn wallpaper_path(&self) -> PathBuf {
        let expanded = shellexpand::tilde(&self.wallpaper_dir);
        PathBuf::from(expanded.as_ref())
    }
}
