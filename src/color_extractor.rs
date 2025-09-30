use anyhow::{Context, Result};
use image::{ImageReader, Rgb};
use std::path::PathBuf;
use crate::{ColorScheme, config::{IroConfig, PaletteStyle}, palette::PaletteGenerator};

pub struct ColorExtractor {
    config: IroConfig,
}

impl ColorExtractor {
    pub fn new() -> Result<Self> {
        let config = IroConfig::load()?;
        Ok(Self { config })
    }

    pub fn extract_colors(&self, image_path: &PathBuf, theme: &str) -> Result<ColorScheme> {
        // Load and resize image for faster processing
        let img = ImageReader::open(image_path)
            .context("Failed to open image")?
            .decode()
            .context("Failed to decode image")?;

        let rgb_img = img.to_rgb8();
        // Use smaller size and faster filter for speed
        let resized = image::imageops::resize(&rgb_img, 128, 128, image::imageops::FilterType::Nearest);

        // Use new palette generator with style
        let style = PaletteStyle::from_name(&self.config.palette.style);
        let palette_gen = PaletteGenerator::new(self.config.palette.diversity_threshold, style);
        let dominant_colors = palette_gen.extract_palette(&resized, self.config.palette.color_count)?;

        // Generate color scheme based on theme
        let color_scheme = match theme {
            "light" => self.generate_light_scheme(dominant_colors, &palette_gen),
            _ => self.generate_dark_scheme(dominant_colors, &palette_gen),
        };

        Ok(color_scheme)
    }

    fn generate_dark_scheme(&self, dominant_colors: Vec<Rgb<u8>>, palette_gen: &PaletteGenerator) -> ColorScheme {
        // Apply style-specific adjustments to colors
        let enhanced: Vec<Rgb<u8>> = dominant_colors
            .iter()
            .map(|c| palette_gen.adjust_with_style(c, false))
            .collect();

        // Generate background based on config
        let background_color = match self.config.theme.dark_background_style.as_str() {
            "extracted" => {
                let bg = palette_gen.generate_background(&enhanced, false);
                format!("#{:02x}{:02x}{:02x}", bg[0], bg[1], bg[2])
            }
            "custom" => {
                self.config.theme.dark_background_custom.as_deref()
                    .unwrap_or("#1e1e2e").to_string()
            }
            _ => "#1e1e2e".to_string(), // pure-dark
        };

        let foreground_color = {
            let bg = self.hex_to_rgb(&background_color).unwrap_or(Rgb([30, 30, 46]));
            let fg = palette_gen.generate_foreground(&bg, false);
            format!("#{:02x}{:02x}{:02x}", fg[0], fg[1], fg[2])
        };

        let mut terminal_colors = Vec::with_capacity(16);

        // Color 0: Dark background
        terminal_colors.push(background_color.clone());

        // Colors 1-7: Use actual extracted colors with minimal forced adjustments
        // Just use the extracted colors directly for more accurate representation
        for i in 0..7 {
            let idx = i % enhanced.len();
            let color = &enhanced[idx];
            terminal_colors.push(format!("#{:02x}{:02x}{:02x}", color[0], color[1], color[2]));
        }

        // Color 7: Light foreground
        terminal_colors.push(foreground_color.clone());

        // Colors 8-15: Brighter versions
        // Color 8 is used by fish shell for autosuggestions - needs good contrast!
        let bright_bg = self.hex_to_rgb(&background_color)
            .map(|c| palette_gen.adjust_brightness(&c, 3.0)) // Much brighter for readability
            .unwrap_or(Rgb([100, 100, 120]));
        terminal_colors.push(format!("#{:02x}{:02x}{:02x}", bright_bg[0], bright_bg[1], bright_bg[2]));

        for i in 1..7 {
            let base_idx = i;
            if let Some(base) = terminal_colors.get(base_idx) {
                if let Ok(rgb) = self.hex_to_rgb(base) {
                    let brighter = palette_gen.adjust_brightness(&rgb, 1.3);
                    terminal_colors.push(format!("#{:02x}{:02x}{:02x}", brighter[0], brighter[1], brighter[2]));
                } else {
                    terminal_colors.push(base.clone());
                }
            }
        }

        let bright_fg = self.hex_to_rgb(&foreground_color)
            .map(|c| palette_gen.adjust_brightness(&c, 1.1))
            .unwrap_or(Rgb([255, 255, 255]));
        terminal_colors.push(format!("#{:02x}{:02x}{:02x}", bright_fg[0], bright_fg[1], bright_fg[2]));

        // Pick most vibrant colors for accent and secondary - avoid cloning
        let mut sorted_by_vibrance: Vec<_> = enhanced
            .iter()
            .map(|c| (c, self.calculate_vibrance(c)))
            .collect();
        sorted_by_vibrance.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let accent_color = sorted_by_vibrance[0].0;
        let secondary_color = sorted_by_vibrance
            .iter()
            .skip(1)
            .map(|(c, _)| *c)
            .find(|c| self.color_distance_simple(c, accent_color) > 80.0)
            .unwrap_or(sorted_by_vibrance[1.min(sorted_by_vibrance.len() - 1)].0);

        // Generate surface color
        let surface_color = self.hex_to_rgb(&background_color)
            .map(|c| palette_gen.adjust_brightness(&c, 1.2))
            .unwrap_or(Rgb([49, 50, 68]));

        ColorScheme {
            background: background_color,
            foreground: foreground_color,
            colors: terminal_colors,
            accent: format!("#{:02x}{:02x}{:02x}", accent_color[0], accent_color[1], accent_color[2]),
            secondary: format!("#{:02x}{:02x}{:02x}", secondary_color[0], secondary_color[1], secondary_color[2]),
            surface: format!("#{:02x}{:02x}{:02x}", surface_color[0], surface_color[1], surface_color[2]),
            error: "#f38ba8".to_string(),
        }
    }

    fn generate_light_scheme(&self, dominant_colors: Vec<Rgb<u8>>, palette_gen: &PaletteGenerator) -> ColorScheme {
        // Apply style-specific adjustments to colors
        let enhanced: Vec<Rgb<u8>> = dominant_colors
            .iter()
            .map(|c| palette_gen.adjust_with_style(c, true))
            .collect();

        // Generate background based on config
        let background_color = match self.config.theme.light_background_style.as_str() {
            "extracted" => {
                let bg = palette_gen.generate_background(&enhanced, true);
                format!("#{:02x}{:02x}{:02x}", bg[0], bg[1], bg[2])
            }
            "custom" => {
                self.config.theme.light_background_custom.as_deref()
                    .unwrap_or("#eff1f5").to_string()
            }
            _ => "#eff1f5".to_string(), // pure-light
        };

        let foreground_color = {
            let bg = self.hex_to_rgb(&background_color).unwrap_or(Rgb([239, 241, 245]));
            let fg = palette_gen.generate_foreground(&bg, true);
            format!("#{:02x}{:02x}{:02x}", fg[0], fg[1], fg[2])
        };

        let mut terminal_colors = Vec::with_capacity(16);

        // Color 0: Light background
        terminal_colors.push(background_color.clone());

        // Colors 1-7: Enhanced colors
        for i in 0..7 {
            let idx = i % enhanced.len();
            let color = &enhanced[idx];
            terminal_colors.push(format!("#{:02x}{:02x}{:02x}", color[0], color[1], color[2]));
        }

        // Color 7: Dark foreground
        terminal_colors.push(foreground_color.clone());

        // Colors 8-15: Brighter/darker variants
        // Color 8 is used by fish shell for autosuggestions - needs good contrast!
        let bright_bg = self.hex_to_rgb(&background_color)
            .map(|c| palette_gen.adjust_brightness(&c, 0.65)) // Much darker for readability
            .unwrap_or(Rgb([140, 145, 160]));
        terminal_colors.push(format!("#{:02x}{:02x}{:02x}", bright_bg[0], bright_bg[1], bright_bg[2]));

        for i in 1..7 {
            if let Some(base) = terminal_colors.get(i) {
                if let Ok(rgb) = self.hex_to_rgb(base) {
                    let darker = palette_gen.adjust_brightness(&rgb, 0.8);
                    terminal_colors.push(format!("#{:02x}{:02x}{:02x}", darker[0], darker[1], darker[2]));
                } else {
                    terminal_colors.push(base.clone());
                }
            }
        }

        let bright_fg = self.hex_to_rgb(&foreground_color)
            .map(|c| palette_gen.adjust_brightness(&c, 0.7))
            .unwrap_or(Rgb([0, 0, 0]));
        terminal_colors.push(format!("#{:02x}{:02x}{:02x}", bright_fg[0], bright_fg[1], bright_fg[2]));

        // Pick most vibrant colors for accent and secondary - avoid cloning
        let mut sorted_by_vibrance: Vec<_> = enhanced
            .iter()
            .map(|c| (c, self.calculate_vibrance(c)))
            .collect();
        sorted_by_vibrance.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let accent_color = sorted_by_vibrance[0].0;
        let secondary_color = sorted_by_vibrance
            .iter()
            .skip(1)
            .map(|(c, _)| *c)
            .find(|c| self.color_distance_simple(c, accent_color) > 80.0)
            .unwrap_or(sorted_by_vibrance[1.min(sorted_by_vibrance.len() - 1)].0);

        // Generate surface color
        let surface_color = self.hex_to_rgb(&background_color)
            .map(|c| palette_gen.adjust_brightness(&c, 0.92))
            .unwrap_or(Rgb([230, 233, 239]));

        ColorScheme {
            background: background_color,
            foreground: foreground_color,
            colors: terminal_colors,
            accent: format!("#{:02x}{:02x}{:02x}", accent_color[0], accent_color[1], accent_color[2]),
            secondary: format!("#{:02x}{:02x}{:02x}", secondary_color[0], secondary_color[1], secondary_color[2]),
            surface: format!("#{:02x}{:02x}{:02x}", surface_color[0], surface_color[1], surface_color[2]),
            error: "#d20f39".to_string(),
        }
    }

    #[inline]
    fn calculate_vibrance(&self, color: &Rgb<u8>) -> f32 {
        let max = color[0].max(color[1]).max(color[2]) as f32;
        let min = color[0].min(color[1]).min(color[2]) as f32;
        if max == 0.0 {
            return 0.0;
        }
        (max - min) / max
    }

    #[inline]
    fn color_distance_simple(&self, c1: &Rgb<u8>, c2: &Rgb<u8>) -> f32 {
        let dr = (c1[0] as i16 - c2[0] as i16) as f32;
        let dg = (c1[1] as i16 - c2[1] as i16) as f32;
        let db = (c1[2] as i16 - c2[2] as i16) as f32;
        (dr * dr + dg * dg + db * db).sqrt()
    }

    fn hex_to_rgb(&self, hex: &str) -> Result<Rgb<u8>> {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16)?;
        let g = u8::from_str_radix(&hex[2..4], 16)?;
        let b = u8::from_str_radix(&hex[4..6], 16)?;
        Ok(Rgb([r, g, b]))
    }
}