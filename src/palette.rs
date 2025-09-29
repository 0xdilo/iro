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

    /// Extract diverse, vibrant colors from an image
    pub fn extract_palette(&self, img: &RgbImage, count: usize) -> Result<Vec<Rgb<u8>>> {
        let mut color_counts: HashMap<(u8, u8, u8), u32> = HashMap::new();

        // Count color frequencies with quantization
        for pixel in img.pixels() {
            // Skip very dark and very bright pixels
            let brightness = (pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32) / 3;
            if brightness < 15 || brightness > 250 {
                continue;
            }

            // Quantize to reduce similar colors
            let quantized = (
                (pixel[0] / 12) * 12,
                (pixel[1] / 12) * 12,
                (pixel[2] / 12) * 12,
            );
            *color_counts.entry(quantized).or_insert(0) += 1;
        }

        // Sort by frequency
        let mut colors: Vec<_> = color_counts.into_iter().collect();
        colors.sort_by(|a, b| b.1.cmp(&a.1));

        // Select diverse colors using HSL color space for better perceptual distance
        let mut selected_colors = Vec::new();

        for ((r, g, b), _) in colors {
            if selected_colors.len() >= count {
                break;
            }

            let color = Rgb([r, g, b]);

            // Check color diversity
            let is_diverse = selected_colors.is_empty()
                || selected_colors
                    .iter()
                    .all(|existing| self.color_distance(&color, existing) > self.diversity_threshold);

            if is_diverse {
                selected_colors.push(color);
            }
        }

        // Fill remaining slots with complementary colors
        while selected_colors.len() < count {
            let complementary = self.generate_complementary_color(&selected_colors);
            selected_colors.push(complementary);
        }

        Ok(selected_colors)
    }

    /// Calculate perceptual color distance using HSL
    fn color_distance(&self, c1: &Rgb<u8>, c2: &Rgb<u8>) -> f32 {
        let rgb1 = Srgb::new(
            c1[0] as f32 / 255.0,
            c1[1] as f32 / 255.0,
            c1[2] as f32 / 255.0,
        );
        let rgb2 = Srgb::new(
            c2[0] as f32 / 255.0,
            c2[1] as f32 / 255.0,
            c2[2] as f32 / 255.0,
        );

        let hsl1: Hsl = rgb1.into_color();
        let hsl2: Hsl = rgb2.into_color();

        // Weighted distance considering hue, saturation, and lightness
        let dh = (hsl1.hue.into_positive_degrees() - hsl2.hue.into_positive_degrees()).abs().min(360.0 - (hsl1.hue.into_positive_degrees() - hsl2.hue.into_positive_degrees()).abs());
        let ds = (hsl1.saturation - hsl2.saturation).abs() * 100.0;
        let dl = (hsl1.lightness - hsl2.lightness).abs() * 100.0;

        // Hue is most important, then saturation, then lightness
        (dh * 0.6 + ds * 0.3 + dl * 0.1)
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

    /// Adjust color saturation
    pub fn adjust_saturation(&self, color: &Rgb<u8>, factor: f32) -> Rgb<u8> {
        let rgb = Srgb::new(
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
        );

        let mut hsl: Hsl = rgb.into_color();
        hsl.saturation = (hsl.saturation * factor).clamp(0.0, 1.0);

        let rgb_out: Srgb = hsl.into_color();
        Rgb([
            (rgb_out.red * 255.0) as u8,
            (rgb_out.green * 255.0) as u8,
            (rgb_out.blue * 255.0) as u8,
        ])
    }

    /// Adjust color with style-specific modifications
    pub fn adjust_with_style(&self, color: &Rgb<u8>, is_light: bool) -> Rgb<u8> {
        let rgb = Srgb::new(
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
        );

        let mut hsl: Hsl = rgb.into_color();

        // Apply style-specific adjustments
        let sat_factor = if is_light {
            self.style.light_saturation
        } else {
            self.style.dark_saturation
        };

        let bright_factor = if is_light {
            self.style.light_brightness
        } else {
            self.style.dark_brightness
        };

        // Apply warmth shift
        if self.style.warmth_shift != 0.0 {
            let hue_shift = self.style.warmth_shift * 30.0; // Max 30 degree shift
            hsl.hue = hsl.hue + hue_shift;
        }

        // Apply saturation and brightness
        hsl.saturation = (hsl.saturation * sat_factor).clamp(0.0, 1.0);
        hsl.lightness = (hsl.lightness * bright_factor).clamp(0.0, 1.0);

        // Apply contrast
        if self.style.contrast != 1.0 {
            let mid = 0.5;
            hsl.lightness = mid + (hsl.lightness - mid) * self.style.contrast;
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

    /// Generate a background color from palette
    pub fn generate_background(&self, colors: &[Rgb<u8>], is_light: bool) -> Rgb<u8> {
        if colors.is_empty() {
            return if is_light {
                Rgb([239, 241, 245]) // Light default
            } else {
                Rgb([30, 30, 46]) // Dark default
            };
        }

        // Find the most muted color (lowest saturation)
        let mut best_color = colors[0];
        let mut lowest_saturation = 1.0;

        for color in colors {
            let rgb = Srgb::new(
                color[0] as f32 / 255.0,
                color[1] as f32 / 255.0,
                color[2] as f32 / 255.0,
            );
            let hsl: Hsl = rgb.into_color();

            if hsl.saturation < lowest_saturation {
                lowest_saturation = hsl.saturation;
                best_color = *color;
            }
        }

        // Adjust the color for background use
        let rgb = Srgb::new(
            best_color[0] as f32 / 255.0,
            best_color[1] as f32 / 255.0,
            best_color[2] as f32 / 255.0,
        );

        let mut hsl: Hsl = rgb.into_color();

        if is_light {
            // Very light, desaturated background
            hsl.lightness = 0.94;
            hsl.saturation = (hsl.saturation * 0.3).min(0.1);
        } else {
            // Very dark, slightly saturated background
            hsl.lightness = 0.12;
            hsl.saturation = (hsl.saturation * 0.4).min(0.15);
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
        if is_light {
            // Dark foreground for light background
            Rgb([76, 79, 105])
        } else {
            // Light foreground for dark background
            Rgb([205, 214, 244])
        }
    }
}