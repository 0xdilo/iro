use anyhow::{Context, Result};
use image::{ImageReader, Rgb, RgbImage};
use std::path::PathBuf;
use std::collections::HashMap;
use crate::ColorScheme;

pub struct ColorExtractor;

impl ColorExtractor {
    pub fn new() -> Self {
        Self
    }

    pub fn extract_colors(&self, image_path: &PathBuf, theme: &str) -> Result<ColorScheme> {
        // Load and resize image for faster processing
        let img = ImageReader::open(image_path)
            .context("Failed to open image")?
            .decode()
            .context("Failed to decode image")?;

        let rgb_img = img.to_rgb8();
        let resized = image::imageops::resize(&rgb_img, 150, 150, image::imageops::FilterType::Lanczos3);

        // Extract dominant colors using k-means-like clustering
        let dominant_colors = self.extract_diverse_colors(&resized, 16)?;

        // Generate color scheme based on theme
        let color_scheme = match theme {
            "light" => self.generate_light_scheme(dominant_colors),
            _ => self.generate_dark_scheme(dominant_colors),
        };

        Ok(color_scheme)
    }

    fn extract_diverse_colors(&self, img: &RgbImage, count: usize) -> Result<Vec<Rgb<u8>>> {
        let mut color_counts: HashMap<(u8, u8, u8), u32> = HashMap::new();

        // Count color frequencies with better quantization
        for pixel in img.pixels() {
            // Skip very dark and very bright pixels for better variety
            let brightness = (pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32) / 3;
            if brightness < 20 || brightness > 245 {
                continue;
            }

            let quantized = (
                (pixel[0] / 16) * 16,
                (pixel[1] / 16) * 16,
                (pixel[2] / 16) * 16,
            );
            *color_counts.entry(quantized).or_insert(0) += 1;
        }

        // Sort by frequency
        let mut colors: Vec<_> = color_counts.into_iter().collect();
        colors.sort_by(|a, b| b.1.cmp(&a.1));

        // Select diverse colors (not just most frequent)
        let mut selected_colors = Vec::new();
        for ((r, g, b), _) in colors {
            if selected_colors.len() >= count {
                break;
            }

            let color = Rgb([r, g, b]);

            // Check if this color is sufficiently different from already selected ones
            let is_diverse = selected_colors.is_empty() || selected_colors.iter().all(|existing| {
                self.color_distance(&color, existing) > 50.0
            });

            if is_diverse {
                selected_colors.push(color);
            }
        }

        // Fill with generated colors if we don't have enough
        while selected_colors.len() < count {
            selected_colors.push(self.generate_complementary_color(&selected_colors));
        }

        Ok(selected_colors)
    }

    fn color_distance(&self, c1: &Rgb<u8>, c2: &Rgb<u8>) -> f32 {
        let dr = (c1[0] as f32 - c2[0] as f32).abs();
        let dg = (c1[1] as f32 - c2[1] as f32).abs();
        let db = (c1[2] as f32 - c2[2] as f32).abs();
        (dr * dr + dg * dg + db * db).sqrt()
    }

    fn generate_complementary_color(&self, existing: &[Rgb<u8>]) -> Rgb<u8> {
        if existing.is_empty() {
            return Rgb([128, 128, 128]);
        }

        let last = existing.last().unwrap();
        Rgb([
            255 - last[0],
            last[1].wrapping_add(80),
            last[2].wrapping_add(120),
        ])
    }

    fn adjust_saturation(&self, color: &Rgb<u8>, factor: f32) -> Rgb<u8> {
        let r = color[0] as f32;
        let g = color[1] as f32;
        let b = color[2] as f32;

        let gray = (r + g + b) / 3.0;

        Rgb([
            (gray + (r - gray) * factor).clamp(0.0, 255.0) as u8,
            (gray + (g - gray) * factor).clamp(0.0, 255.0) as u8,
            (gray + (b - gray) * factor).clamp(0.0, 255.0) as u8,
        ])
    }

    fn adjust_brightness(&self, color: &Rgb<u8>, factor: f32) -> Rgb<u8> {
        Rgb([
            ((color[0] as f32 * factor).clamp(0.0, 255.0)) as u8,
            ((color[1] as f32 * factor).clamp(0.0, 255.0)) as u8,
            ((color[2] as f32 * factor).clamp(0.0, 255.0)) as u8,
        ])
    }

    fn generate_dark_scheme(&self, dominant_colors: Vec<Rgb<u8>>) -> ColorScheme {
        // Use extracted colors to build a vibrant palette
        let base_colors = &dominant_colors;

        // Enhance saturation for more vibrant colors
        let enhanced: Vec<Rgb<u8>> = base_colors.iter()
            .map(|c| self.adjust_saturation(c, 1.4))
            .collect();

        let mut terminal_colors = Vec::new();

        // Color 0: Dark background
        terminal_colors.push("#1e1e2e".to_string());

        // Colors 1-7: Use actual extracted colors with minimal forced adjustments
        // Just use the extracted colors directly for more accurate representation
        for i in 0..7 {
            let idx = i % enhanced.len();
            let color = &enhanced[idx];
            terminal_colors.push(format!("#{:02x}{:02x}{:02x}", color[0], color[1], color[2]));
        }

        // Color 7: Light foreground
        terminal_colors.push("#cdd6f4".to_string());

        // Colors 8-15: Brighter versions
        terminal_colors.push("#45475a".to_string()); // Bright black
        for i in 1..7 {
            let base_idx = i;
            if let Some(base) = terminal_colors.get(base_idx) {
                if let Ok(rgb) = self.hex_to_rgb(base) {
                    let brighter = self.adjust_brightness(&rgb, 1.2);
                    terminal_colors.push(format!("#{:02x}{:02x}{:02x}", brighter[0], brighter[1], brighter[2]));
                } else {
                    terminal_colors.push(base.clone());
                }
            }
        }
        terminal_colors.push("#ffffff".to_string()); // Bright white

        // Pick accent and secondary - avoid green bias
        // Choose colors that are NOT predominantly green
        let mut accent_color = &enhanced[0];
        let mut secondary_color = &enhanced[std::cmp::min(1, enhanced.len() - 1)];

        // Find first non-green color for accent
        for color in enhanced.iter() {
            let is_green = color[1] > color[0] + 30 && color[1] > color[2] + 30;
            if !is_green {
                accent_color = color;
                break;
            }
        }

        // Find second distinct non-green color for secondary
        for color in enhanced.iter() {
            let is_green = color[1] > color[0] + 30 && color[1] > color[2] + 30;
            let too_similar = self.color_distance(color, accent_color) < 80.0;
            if !is_green && !too_similar {
                secondary_color = color;
                break;
            }
        }

        ColorScheme {
            background: "#1e1e2e".to_string(),
            foreground: "#cdd6f4".to_string(),
            colors: terminal_colors,
            accent: format!("#{:02x}{:02x}{:02x}", accent_color[0], accent_color[1], accent_color[2]),
            secondary: format!("#{:02x}{:02x}{:02x}", secondary_color[0], secondary_color[1], secondary_color[2]),
            surface: "#313244".to_string(),
            error: "#f38ba8".to_string(),
        }
    }

    fn generate_light_scheme(&self, dominant_colors: Vec<Rgb<u8>>) -> ColorScheme {
        let enhanced: Vec<Rgb<u8>> = dominant_colors.iter()
            .map(|c| self.adjust_saturation(c, 1.2))
            .map(|c| self.adjust_brightness(&c, 0.8))
            .collect();

        let mut terminal_colors = Vec::new();

        terminal_colors.push("#eff1f5".to_string()); // Light background

        for i in 0..7 {
            let idx = i % enhanced.len();
            let color = &enhanced[idx];
            terminal_colors.push(format!("#{:02x}{:02x}{:02x}", color[0], color[1], color[2]));
        }

        terminal_colors.push("#4c4f69".to_string()); // Dark foreground

        // Bright versions
        terminal_colors.push("#bcc0cc".to_string());
        for i in 1..7 {
            if let Some(base) = terminal_colors.get(i) {
                terminal_colors.push(base.clone());
            }
        }
        terminal_colors.push("#000000".to_string());

        let accent = &enhanced[0];
        let secondary = &enhanced[std::cmp::min(2, enhanced.len() - 1)];

        ColorScheme {
            background: "#eff1f5".to_string(),
            foreground: "#4c4f69".to_string(),
            colors: terminal_colors,
            accent: format!("#{:02x}{:02x}{:02x}", accent[0], accent[1], accent[2]),
            secondary: format!("#{:02x}{:02x}{:02x}", secondary[0], secondary[1], secondary[2]),
            surface: "#e6e9ef".to_string(),
            error: "#d20f39".to_string(),
        }
    }

    fn hex_to_rgb(&self, hex: &str) -> Result<Rgb<u8>> {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16)?;
        let g = u8::from_str_radix(&hex[2..4], 16)?;
        let b = u8::from_str_radix(&hex[4..6], 16)?;
        Ok(Rgb([r, g, b]))
    }
}