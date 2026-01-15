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

        // Generate intelligent terminal colors based on hue mapping
        let terminal_colors = self.generate_terminal_colors(&enhanced, &background_color, &foreground_color, palette_gen, false);

        // Pick most vibrant colors for accent and secondary
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

        // Generate intelligent terminal colors based on hue mapping
        let terminal_colors = self.generate_terminal_colors(&enhanced, &background_color, &foreground_color, palette_gen, true);

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

    /// Generate terminal colors (0-15) with intelligent hue mapping
    fn generate_terminal_colors(
        &self,
        colors: &[Rgb<u8>],
        background: &str,
        foreground: &str,
        palette_gen: &PaletteGenerator,
        is_light: bool,
    ) -> Vec<String> {
        use palette::{Hsl, IntoColor, Srgb};

        let mut terminal_colors = Vec::with_capacity(16);

        // Color 0: Background
        terminal_colors.push(background.to_string());

        // Define hue ranges for terminal colors
        // Red: 0-30°, Yellow: 30-90°, Green: 90-150°, Cyan: 150-210°, Blue: 210-270°, Magenta: 270-360°
        let hue_ranges = [
            (345.0, 30.0),   // Red (wraps around 0)
            (30.0, 90.0),    // Yellow
            (90.0, 150.0),   // Green
            (150.0, 210.0),  // Cyan
            (210.0, 270.0),  // Blue
            (270.0, 345.0),  // Magenta
        ];

        // Find best color for each hue range
        let mut base_colors = Vec::with_capacity(6);
        for (hue_start, hue_end) in hue_ranges.iter() {
            let best_color = self.find_color_in_hue_range(colors, *hue_start, *hue_end);

            let color = if let Some(c) = best_color {
                c
            } else {
                // Generate synthetic color in this hue range
                self.generate_color_at_hue((*hue_start + *hue_end) / 2.0, is_light)
            };

            base_colors.push(color);
        }

        // Colors 1-6: Red, Yellow, Green, Cyan, Blue, Magenta
        for color in base_colors.iter() {
            // Boost saturation and adjust lightness for better visibility
            let rgb = Srgb::new(
                color[0] as f32 / 255.0,
                color[1] as f32 / 255.0,
                color[2] as f32 / 255.0,
            );
            let mut hsl: Hsl = rgb.into_color();

            if is_light {
                hsl.saturation = (hsl.saturation * 1.3).min(0.85);
                hsl.lightness = (hsl.lightness * 0.75).clamp(0.35, 0.55);
            } else {
                hsl.saturation = (hsl.saturation * 1.4).min(0.9);
                hsl.lightness = (hsl.lightness * 1.15).clamp(0.50, 0.70);
            }

            let rgb_out: Srgb = hsl.into_color();
            terminal_colors.push(format!(
                "#{:02x}{:02x}{:02x}",
                (rgb_out.red * 255.0) as u8,
                (rgb_out.green * 255.0) as u8,
                (rgb_out.blue * 255.0) as u8
            ));
        }

        // Color 7: Foreground
        terminal_colors.push(foreground.to_string());

        // Color 8: Bright black (comment color - needs good contrast)
        let bright_bg = self.hex_to_rgb(background)
            .map(|c| {
                if is_light {
                    palette_gen.adjust_brightness(&c, 0.65)
                } else {
                    palette_gen.adjust_brightness(&c, 3.0)
                }
            })
            .unwrap_or(if is_light { Rgb([140, 145, 160]) } else { Rgb([100, 100, 120]) });
        terminal_colors.push(format!("#{:02x}{:02x}{:02x}", bright_bg[0], bright_bg[1], bright_bg[2]));

        // Colors 9-14: Bright versions of 1-6
        for i in 1..=6 {
            if let Ok(rgb) = self.hex_to_rgb(&terminal_colors[i]) {
                let rgb_srgb = Srgb::new(
                    rgb[0] as f32 / 255.0,
                    rgb[1] as f32 / 255.0,
                    rgb[2] as f32 / 255.0,
                );
                let mut hsl: Hsl = rgb_srgb.into_color();

                if is_light {
                    // Darker and more saturated for light mode
                    hsl.saturation = (hsl.saturation * 1.15).min(0.95);
                    hsl.lightness = (hsl.lightness * 0.85).clamp(0.30, 0.50);
                } else {
                    // Brighter and more saturated for dark mode
                    hsl.saturation = (hsl.saturation * 1.2).min(0.95);
                    hsl.lightness = (hsl.lightness * 1.25).clamp(0.60, 0.85);
                }

                let rgb_out: Srgb = hsl.into_color();
                terminal_colors.push(format!(
                    "#{:02x}{:02x}{:02x}",
                    (rgb_out.red * 255.0) as u8,
                    (rgb_out.green * 255.0) as u8,
                    (rgb_out.blue * 255.0) as u8
                ));
            } else {
                terminal_colors.push(terminal_colors[i].clone());
            }
        }

        // Color 15: Bright white
        let bright_fg = self.hex_to_rgb(foreground)
            .map(|c| {
                if is_light {
                    palette_gen.adjust_brightness(&c, 0.7)
                } else {
                    palette_gen.adjust_brightness(&c, 1.1)
                }
            })
            .unwrap_or(if is_light { Rgb([0, 0, 0]) } else { Rgb([255, 255, 255]) });
        terminal_colors.push(format!("#{:02x}{:02x}{:02x}", bright_fg[0], bright_fg[1], bright_fg[2]));

        terminal_colors
    }

    /// Find the best color in a hue range
    fn find_color_in_hue_range(&self, colors: &[Rgb<u8>], hue_start: f32, hue_end: f32) -> Option<Rgb<u8>> {
        use palette::{Hsl, IntoColor, Srgb};

        let mut best_color: Option<(Rgb<u8>, f32)> = None;

        for color in colors {
            let rgb = Srgb::new(
                color[0] as f32 / 255.0,
                color[1] as f32 / 255.0,
                color[2] as f32 / 255.0,
            );
            let hsl: Hsl = rgb.into_color();
            let hue = hsl.hue.into_positive_degrees();

            // Check if hue is in range (handle wraparound)
            let in_range = if hue_start > hue_end {
                // Wraparound case (e.g., red: 345-30)
                hue >= hue_start || hue <= hue_end
            } else {
                hue >= hue_start && hue <= hue_end
            };

            if in_range {
                let score = hsl.saturation * self.calculate_vibrance(color);
                if best_color.is_none() || score > best_color.unwrap().1 {
                    best_color = Some((*color, score));
                }
            }
        }

        best_color.map(|(c, _)| c)
    }

    /// Generate a synthetic color at a specific hue
    fn generate_color_at_hue(&self, hue: f32, is_light: bool) -> Rgb<u8> {
        use palette::{Hsl, IntoColor, Srgb};

        // Boost saturation for pink/magenta hues (270-345) to make them cuter
        let is_pink_range = hue >= 270.0 && hue <= 345.0;
        let sat_boost: f32 = if is_pink_range { 0.12 } else { 0.0 };

        let hsl = if is_light {
            Hsl::new(hue, (0.70_f32 + sat_boost).min(0.85), 0.50)
        } else {
            Hsl::new(hue, (0.78_f32 + sat_boost).min(0.92), 0.65)
        };

        let rgb: Srgb = hsl.into_color();
        Rgb([
            (rgb.red * 255.0) as u8,
            (rgb.green * 255.0) as u8,
            (rgb.blue * 255.0) as u8,
        ])
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