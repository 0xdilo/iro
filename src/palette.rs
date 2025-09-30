use anyhow::Result;
use image::{Rgb, RgbImage};
use palette::{Hsl, IntoColor, Srgb};
use std::collections::HashMap;
use crate::config::PaletteStyle;

pub struct PaletteGenerator {
    diversity_threshold: f32,
    style: PaletteStyle,
}

impl PaletteGenerator {
    pub fn new(diversity_threshold: f32, style: PaletteStyle) -> Self {
        Self {
            diversity_threshold,
            style,
        }
    }

    /// Extract diverse colors from an image
    pub fn extract_palette(&self, img: &RgbImage, count: usize) -> Result<Vec<Rgb<u8>>> {
        let mut color_counts: HashMap<(u8, u8, u8), u32> = HashMap::with_capacity(4096);

        // Count color frequencies with quantization - optimized
        for pixel in img.pixels() {
            // Skip very dark and very bright pixels for better palette
            let brightness = (pixel[0] as u16 + pixel[1] as u16 + pixel[2] as u16) / 3;
            if !(20..=240).contains(&brightness) {
                continue;
            }

            // Quantize to 16-step intervals for performance
            let quantized = (
                (pixel[0] >> 4) << 4,
                (pixel[1] >> 4) << 4,
                (pixel[2] >> 4) << 4,
            );
            *color_counts.entry(quantized).or_insert(0) += 1;
        }

        // Sort by frequency
        let mut colors: Vec<_> = color_counts.into_iter().collect();
        colors.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        // Select diverse colors - optimized
        let mut selected_colors = Vec::with_capacity(count);

        for ((r, g, b), _) in colors.iter().take(count * 3) {
            if selected_colors.len() >= count {
                break;
            }

            let color = Rgb([*r, *g, *b]);

            // Check diversity only against existing colors
            if selected_colors.is_empty()
                || selected_colors.iter().all(|existing| {
                    self.color_distance(&color, existing) > self.diversity_threshold
                }) {
                selected_colors.push(color);
            }
        }

        // Fill remaining with complementary if needed
        while selected_colors.len() < count {
            selected_colors.push(self.generate_complementary_color(&selected_colors));
        }

        Ok(selected_colors)
    }

    /// Calculate color distance - simplified for speed
    #[inline]
    fn color_distance(&self, c1: &Rgb<u8>, c2: &Rgb<u8>) -> f32 {
        // Simple euclidean distance in RGB space - much faster than HSL conversion
        let dr = (c1[0] as i16 - c2[0] as i16).abs() as f32;
        let dg = (c1[1] as i16 - c2[1] as i16).abs() as f32;
        let db = (c1[2] as i16 - c2[2] as i16).abs() as f32;

        // Weighted for human perception (green more sensitive)
        dr * 0.3 + dg * 0.59 + db * 0.11
    }

    /// Generate a complementary color
    fn generate_complementary_color(&self, existing: &[Rgb<u8>]) -> Rgb<u8> {
        if existing.is_empty() {
            return Rgb([128, 128, 128]);
        }

        let last = existing.last().unwrap();
        let rgb = Srgb::new(
            last[0] as f32 / 255.0,
            last[1] as f32 / 255.0,
            last[2] as f32 / 255.0,
        );

        let mut hsl: Hsl = rgb.into_color();

        // Rotate hue by ~180 degrees for complementary
        hsl.hue = hsl.hue + 180.0;

        // Adjust saturation and lightness slightly
        hsl.saturation = (hsl.saturation + 0.2).min(1.0);
        hsl.lightness = 0.5;

        let rgb_out: Srgb = hsl.into_color();
        Rgb([
            (rgb_out.red * 255.0) as u8,
            (rgb_out.green * 255.0) as u8,
            (rgb_out.blue * 255.0) as u8,
        ])
    }

    /// Adjust color with style-specific modifications
    #[inline]
    pub fn adjust_with_style(&self, color: &Rgb<u8>, is_light: bool) -> Rgb<u8> {
        let rgb = Srgb::new(
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
        );

        let mut hsl: Hsl = rgb.into_color();

        // Apply style-specific adjustments
        let (sat_factor, bright_factor) = if is_light {
            (self.style.light_saturation, self.style.light_brightness)
        } else {
            (self.style.dark_saturation, self.style.dark_brightness)
        };

        // Apply warmth shift if needed
        if self.style.warmth_shift.abs() > 0.001 {
            hsl.hue = hsl.hue + (self.style.warmth_shift * 30.0);
        }

        // Apply saturation and brightness
        hsl.saturation = (hsl.saturation * sat_factor).clamp(0.0, 1.0);
        hsl.lightness = (hsl.lightness * bright_factor).clamp(0.0, 1.0);

        // Apply contrast if needed
        if (self.style.contrast - 1.0).abs() > 0.001 {
            hsl.lightness = 0.5 + (hsl.lightness - 0.5) * self.style.contrast;
            hsl.lightness = hsl.lightness.clamp(0.0, 1.0);
        }

        let rgb_out: Srgb = hsl.into_color();
        Rgb([
            (rgb_out.red * 255.0) as u8,
            (rgb_out.green * 255.0) as u8,
            (rgb_out.blue * 255.0) as u8,
        ])
    }

    /// Adjust color brightness
    pub fn adjust_brightness(&self, color: &Rgb<u8>, factor: f32) -> Rgb<u8> {
        let rgb = Srgb::new(
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
        );

        let mut hsl: Hsl = rgb.into_color();
        hsl.lightness = (hsl.lightness * factor).clamp(0.0, 1.0);

        let rgb_out: Srgb = hsl.into_color();
        Rgb([
            (rgb_out.red * 255.0) as u8,
            (rgb_out.green * 255.0) as u8,
            (rgb_out.blue * 255.0) as u8,
        ])
    }

    /// Generate a background color from palette - intelligently based on image tone
    pub fn generate_background(&self, colors: &[Rgb<u8>], is_light: bool) -> Rgb<u8> {
        if colors.is_empty() {
            return if is_light {
                Rgb([239, 241, 245]) // Light default
            } else {
                Rgb([30, 30, 46]) // Dark default
            };
        }

        // Calculate average brightness and saturation of the palette
        let mut total_lightness = 0.0;
        let mut total_saturation = 0.0;
        let mut hue_accumulator = (0.0, 0.0); // (sin, cos) for circular mean

        for color in colors {
            let rgb = Srgb::new(
                color[0] as f32 / 255.0,
                color[1] as f32 / 255.0,
                color[2] as f32 / 255.0,
            );
            let hsl: Hsl = rgb.into_color();

            total_lightness += hsl.lightness;
            total_saturation += hsl.saturation;

            // Accumulate hue using circular statistics
            let hue_rad = hsl.hue.into_positive_degrees() * std::f32::consts::PI / 180.0;
            hue_accumulator.0 += hue_rad.sin() * hsl.saturation;
            hue_accumulator.1 += hue_rad.cos() * hsl.saturation;
        }

        let avg_lightness = total_lightness / colors.len() as f32;
        let avg_saturation = total_saturation / colors.len() as f32;

        // Calculate dominant hue
        let dominant_hue = if hue_accumulator.0.abs() < 0.001 && hue_accumulator.1.abs() < 0.001 {
            0.0 // Neutral/grey
        } else {
            hue_accumulator.0.atan2(hue_accumulator.1) * 180.0 / std::f32::consts::PI
        };

        // Generate background based on palette characteristics
        let mut hsl = Hsl::new(dominant_hue, 0.0, 0.0);

        if is_light {
            // Light mode: very light, subtle tint from dominant hue
            hsl.lightness = 0.91 + (avg_lightness * 0.06); // 0.91-0.97 range - more variation
            hsl.saturation = (avg_saturation * 0.25).min(0.12); // More visible tint
        } else {
            // Dark mode: adapt to wallpaper brightness - brighter walls get darker bg, dark walls get lighter bg
            hsl.lightness = 0.06 + (1.0 - avg_lightness) * 0.12; // 0.06-0.18 range - much wider!
            hsl.saturation = (avg_saturation * 0.4).min(0.18); // More saturated for visible tint
        }

        let rgb_out: Srgb = hsl.into_color();
        Rgb([
            (rgb_out.red * 255.0) as u8,
            (rgb_out.green * 255.0) as u8,
            (rgb_out.blue * 255.0) as u8,
        ])
    }

    /// Generate a foreground color that contrasts with background
    pub fn generate_foreground(&self, background: &Rgb<u8>, is_light: bool) -> Rgb<u8> {
        // Extract hue from background
        let bg_rgb = Srgb::new(
            background[0] as f32 / 255.0,
            background[1] as f32 / 255.0,
            background[2] as f32 / 255.0,
        );
        let bg_hsl: Hsl = bg_rgb.into_color();

        // Create foreground with same hue but high contrast
        let mut fg_hsl = bg_hsl;

        if is_light {
            // Dark text on light background
            fg_hsl.lightness = 0.25;
            fg_hsl.saturation = (bg_hsl.saturation * 0.5).min(0.15); // Subtle tint
        } else {
            // Light text on dark background
            fg_hsl.lightness = 0.85;
            fg_hsl.saturation = (bg_hsl.saturation * 0.4).min(0.12); // Subtle tint
        }

        let fg_rgb: Srgb = fg_hsl.into_color();
        Rgb([
            (fg_rgb.red * 255.0) as u8,
            (fg_rgb.green * 255.0) as u8,
            (fg_rgb.blue * 255.0) as u8,
        ])
    }
}