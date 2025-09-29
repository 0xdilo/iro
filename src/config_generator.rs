use anyhow::{Context, Result};
use std::path::PathBuf;
use crate::{ColorScheme, template_engine::TemplateEngine};

pub struct ConfigGenerator {
    template_engine: TemplateEngine,
    config_dir: PathBuf,
}

impl ConfigGenerator {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?;
        
        let template_engine = TemplateEngine::new()?;
        
        // Create default templates if they don't exist
        template_engine.create_default_templates()?;
        
        Ok(Self {
            template_engine,
            config_dir,
        })
    }

    pub fn generate_configs(&self, color_scheme: &ColorScheme) -> Result<()> {
        println!("📝 Generating configuration files...");
        
        // Generate Hyprland config
        self.generate_hyprland_config(color_scheme)
            .context("Failed to generate Hyprland config")?;
        
        // Generate Waybar config  
        self.generate_waybar_config(color_scheme)
            .context("Failed to generate Waybar config")?;
        
        // Generate Kitty config
        self.generate_kitty_config(color_scheme)
            .context("Failed to generate Kitty config")?;
        
        // Generate shell colors
        self.generate_shell_colors(color_scheme)
            .context("Failed to generate shell colors")?;
        
        println!("  ✓ Generated all configuration files");
        Ok(())
    }

    fn generate_hyprland_config(&self, color_scheme: &ColorScheme) -> Result<()> {
        let hyprland_dir = self.config_dir.join("hypr");
        let config_path = hyprland_dir.join("hyprland.conf");
        
        // Backup original config if it exists and no backup exists
        self.backup_config(&config_path)?;
        
        // Read the current config to preserve non-color settings
        let current_config = if config_path.exists() {
            std::fs::read_to_string(&config_path).unwrap_or_default()
        } else {
            String::new()
        };
        
        // Generate the color section
        let color_section = self.generate_hyprland_colors(color_scheme)?;
        
        // Replace or add color section in the config
        let updated_config = if current_config.contains("# Dynamic Color scheme") {
            // Replace existing dynamic colors
            self.replace_section(&current_config, "# Dynamic Color scheme", "# General settings", &color_section)
        } else if current_config.contains("# Color scheme") {
            // Replace old static color scheme
            self.replace_section(&current_config, "# Color scheme", "# General settings", &color_section)
        } else {
            // Add color section after input section
            self.insert_after_section(&current_config, "}", &color_section)
        };
        
        std::fs::write(&config_path, updated_config)
            .context("Failed to write Hyprland config")?;
        
        println!("  ✓ Updated Hyprland colors");
        Ok(())
    }

    fn generate_waybar_config(&self, color_scheme: &ColorScheme) -> Result<()> {
        let waybar_dir = self.config_dir.join("waybar");
        let style_path = waybar_dir.join("style.css");
        
        // Backup original style
        self.backup_config(&style_path)?;
        
        // Generate new CSS with dynamic colors
        let rendered_css = self.template_engine.render_template("waybar.css", color_scheme)?;
        
        std::fs::write(&style_path, rendered_css)
            .context("Failed to write Waybar style")?;
        
        println!("  ✓ Updated Waybar colors");
        Ok(())
    }

    fn generate_kitty_config(&self, color_scheme: &ColorScheme) -> Result<()> {
        let kitty_dir = self.config_dir.join("kitty");
        let config_path = kitty_dir.join("kitty.conf");
        
        // Backup original config
        self.backup_config(&config_path)?;
        
        // Read current config to preserve non-color settings
        let current_config = if config_path.exists() {
            std::fs::read_to_string(&config_path).unwrap_or_default()
        } else {
            String::new()
        };
        
        // Generate color section
        let color_section = self.generate_kitty_colors(color_scheme)?;
        
        // Replace or add color section
        let updated_config = if current_config.contains("# DYNAMIC COLOR SCHEME") {
            // Replace existing dynamic colors
            self.replace_section(&current_config, 
                "# ═══════════════════════════════════════════════════════════════════\n# DYNAMIC COLOR SCHEME", 
                "# ═══════════════════════════════════════════════════════════════════\n# TAB BAR", 
                &color_section)
        } else if current_config.contains("# ROSE PINE DAWN x TOKYO NIGHT COLOR SCHEME") {
            // Replace old static color scheme  
            self.replace_section(&current_config,
                "# ═══════════════════════════════════════════════════════════════════\n# ROSE PINE DAWN x TOKYO NIGHT COLOR SCHEME",
                "# ═══════════════════════════════════════════════════════════════════\n# TAB BAR",
                &color_section)
        } else {
            // Add color section before tab bar or at end
            if current_config.contains("# TAB BAR") {
                self.replace_section(&current_config, "# ═══════════════════════════════════════════════════════════════════\n# TAB BAR", "", &format!("{}\n\n# ═══════════════════════════════════════════════════════════════════\n# TAB BAR", color_section))
            } else {
                format!("{}\n\n{}", current_config, color_section)
            }
        };
        
        std::fs::write(&config_path, updated_config)
            .context("Failed to write Kitty config")?;
        
        println!("  ✓ Updated Kitty colors");
        Ok(())
    }

    fn generate_shell_colors(&self, color_scheme: &ColorScheme) -> Result<()> {
        let shell_colors = self.template_engine.render_template("shell_colors.sh", color_scheme)?;
        
        // Write to iro config directory
        let iro_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("iro");
        std::fs::create_dir_all(&iro_dir)?;

        let shell_colors_path = iro_dir.join("colors.sh");
        std::fs::write(&shell_colors_path, shell_colors)
            .context("Failed to write shell colors")?;
        
        // Make it executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&shell_colors_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&shell_colors_path, perms)?;
        }
        
        println!("  ✓ Generated shell colors (source ~/.config/iro/colors.sh)");
        Ok(())
    }

    fn generate_hyprland_colors(&self, color_scheme: &ColorScheme) -> Result<String> {
        Ok(format!(r#"
# Dynamic Color scheme - Generated by iro
$red = rgb({})
$blue = rgb({})
$yellow = rgb({})
$magenta = rgb({})
$accent = rgb({})
$secondary = rgb({})
$text = rgb({})
$surface = rgb({})
$surface0 = rgb({})
$base = rgb({})
$mantle = rgb(292c3c)
$crust = rgb(232634)
$error = rgb({})
"#,
            color_scheme.colors.get(1).unwrap_or(&"#e78284".to_string()).trim_start_matches('#'),
            color_scheme.colors.get(4).unwrap_or(&"#8caaee".to_string()).trim_start_matches('#'),
            color_scheme.colors.get(3).unwrap_or(&"#e5c890".to_string()).trim_start_matches('#'),
            color_scheme.colors.get(5).unwrap_or(&"#ca9ee6".to_string()).trim_start_matches('#'),
            color_scheme.accent.trim_start_matches('#'),
            color_scheme.secondary.trim_start_matches('#'),
            color_scheme.foreground.trim_start_matches('#'),
            color_scheme.surface.trim_start_matches('#'),
            color_scheme.surface.trim_start_matches('#'), // surface0 - same as surface
            color_scheme.background.trim_start_matches('#'),
            color_scheme.error.trim_start_matches('#'),
        ))
    }

    fn generate_kitty_colors(&self, color_scheme: &ColorScheme) -> Result<String> {
        let mut color_lines = Vec::new();
        color_lines.push("# ═══════════════════════════════════════════════════════════════════".to_string());
        color_lines.push("# DYNAMIC COLOR SCHEME - Generated by iro".to_string());
        color_lines.push("# ═══════════════════════════════════════════════════════════════════".to_string());
        color_lines.push("".to_string());
        color_lines.push(format!("# Background and foreground"));
        color_lines.push(format!("foreground            {}", color_scheme.foreground));
        color_lines.push(format!("background            {}", color_scheme.background));
        color_lines.push(format!("selection_foreground  {}", color_scheme.background));
        color_lines.push(format!("selection_background  {}", color_scheme.accent));
        color_lines.push("".to_string());
        color_lines.push(format!("# Cursor colors"));
        color_lines.push(format!("cursor                {}", color_scheme.accent));
        color_lines.push(format!("cursor_text_color     {}", color_scheme.background));
        color_lines.push("".to_string());
        color_lines.push(format!("# Terminal colors (0-15)"));
        
        for (i, color) in color_scheme.colors.iter().enumerate() {
            color_lines.push(format!("color{}   {}", i, color));
        }
        
        color_lines.push("".to_string());
        color_lines.push(format!("# Tab colors"));
        color_lines.push(format!("active_tab_foreground   {}", color_scheme.background));
        color_lines.push(format!("active_tab_background   {}", color_scheme.accent));
        color_lines.push(format!("inactive_tab_foreground {}", color_scheme.secondary));
        color_lines.push(format!("inactive_tab_background {}", color_scheme.background));
        
        color_lines.push("".to_string());
        color_lines.push("".to_string());
        color_lines.push(format!("# Window borders"));
        color_lines.push(format!("active_border_color   {}", color_scheme.accent));
        color_lines.push(format!("inactive_border_color {}", color_scheme.surface));
        color_lines.push(format!("bell_border_color     {}", color_scheme.error));
        color_lines.push("".to_string());
        
        Ok(color_lines.join("\n"))
    }

    fn backup_config(&self, config_path: &PathBuf) -> Result<()> {
        if config_path.exists() {
            let backup_path = config_path.with_extension("conf.iro.bak");
            if !backup_path.exists() {
                std::fs::copy(config_path, &backup_path)
                    .with_context(|| format!("Failed to backup {}", config_path.display()))?;
                println!("  💾 Backed up original config to {}", backup_path.display());
            }
        }
        Ok(())
    }

    fn replace_section(&self, content: &str, start_marker: &str, end_marker: &str, replacement: &str) -> String {
        if let Some(start_pos) = content.find(start_marker) {
            let before = &content[..start_pos];
            
            if end_marker.is_empty() {
                format!("{}{}", before, replacement)
            } else if let Some(end_pos) = content[start_pos..].find(end_marker) {
                let after = &content[start_pos + end_pos..];
                format!("{}{}{}", before, replacement, after)
            } else {
                format!("{}{}", before, replacement)
            }
        } else {
            content.to_string()
        }
    }

    fn insert_after_section(&self, content: &str, marker: &str, insertion: &str) -> String {
        if let Some(pos) = content.find(marker) {
            let end_pos = pos + marker.len();
            let before = &content[..end_pos];
            let after = &content[end_pos..];
            format!("{}\n{}{}", before, insertion, after)
        } else {
            format!("{}\n{}", content, insertion)
        }
    }
}