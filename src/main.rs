use anyhow::{Context, Result};
use clap::{Arg, Command};
use image::{ImageReader, Rgb};
use std::path::PathBuf;
use std::collections::HashMap;

mod color_extractor;
mod template_engine;
mod config_generator;
mod gui;

use color_extractor::ColorExtractor;
use template_engine::TemplateEngine;
use config_generator::ConfigGenerator;

#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub background: String,
    pub foreground: String,
    pub colors: Vec<String>, // 16 terminal colors
    pub accent: String,
    pub secondary: String,
    pub surface: String,
    pub error: String,
}

fn main() -> Result<()> {
    let matches = Command::new("iro")
        .version("0.1.0")
        .about("Fast, elegant wallpaper-based color scheme generator for Hyprland")
        .arg(
            Arg::new("wallpaper")
                .help("Path to wallpaper image")
                .required(false)
                .index(1)
        )
        .arg(
            Arg::new("theme")
                .short('t')
                .long("theme")
                .value_name("THEME")
                .help("Color scheme theme (dark, light)")
                .default_value("dark")
        )
        .arg(
            Arg::new("reload")
                .short('r')
                .long("reload")
                .help("Reload applications after generating configs")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("gui")
                .short('g')
                .long("gui")
                .help("Open GUI mode to select wallpaper")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("init")
                .long("init")
                .help("Initialize iro: setup directories, copy templates, and integrate with shell")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    let theme = matches.get_one::<String>("theme").unwrap();
    let should_reload = matches.get_flag("reload");
    let gui_mode = matches.get_flag("gui");
    let init_mode = matches.get_flag("init");

    // Handle init mode
    if init_mode {
        return run_init();
    }

    // Handle GUI mode
    let wallpaper_path = if gui_mode {
        open_wallpaper_picker()?
    } else {
        match matches.get_one::<String>("wallpaper") {
            Some(path) => PathBuf::from(path),
            None => {
                eprintln!("Error: Wallpaper path is required (or use --gui mode)");
                std::process::exit(1);
            }
        }
    };

    println!("🎨 iro - Generating color scheme from: {}", wallpaper_path.display());
    
    // Extract colors from wallpaper
    let extractor = ColorExtractor::new();
    let color_scheme = extractor.extract_colors(&wallpaper_path, theme)?;
    
    println!("✨ Extracted color scheme:");
    print_color_scheme(&color_scheme);
    
    // Generate configurations
    let config_gen = ConfigGenerator::new()?;
    config_gen.generate_configs(&color_scheme)?;
    
    // In GUI mode, always reload. Otherwise, check the flag
    if gui_mode || should_reload {
        println!("🔄 Reloading applications...");
        reload_applications()?;
    }
    
    // Set wallpaper if in GUI mode
    if gui_mode {
        set_wallpaper(&wallpaper_path)?;
    }
    
    println!("✅ Color scheme applied successfully!");
    Ok(())
}

fn print_color_scheme(scheme: &ColorScheme) {
    println!("  Background: {}", scheme.background);
    println!("  Foreground: {}", scheme.foreground);
    println!("  Accent: {}", scheme.accent);
    println!("  Secondary: {}", scheme.secondary);
    println!("  Colors: {:?}", &scheme.colors[0..8]);
}

fn reload_applications() -> Result<()> {
    // Restart waybar
    std::process::Command::new("pkill")
        .arg("waybar")
        .output()
        .ok();

    std::process::Command::new("waybar")
        .spawn()
        .context("Failed to start waybar")?;

    // Reload hyprland config
    std::process::Command::new("hyprctl")
        .args(["reload"])
        .output()
        .context("Failed to reload hyprland")?;

    println!("  ✓ Restarted waybar and reloaded hyprland");
    Ok(())
}

fn open_wallpaper_picker() -> Result<PathBuf> {
    // Launch the Rust GUI
    println!("🎨 Launching iro GUI viewer...");
    gui::launch_gui()?;
    
    // GUI handles everything, so we can exit
    std::process::exit(0);
}

fn set_wallpaper(wallpaper_path: &PathBuf) -> Result<()> {
    // Create hyprpaper config
    let temp_config = "/tmp/iro_hyprpaper.conf";
    let config_content = format!(
        "preload = {}\nwallpaper = ,{}\n",
        wallpaper_path.display(),
        wallpaper_path.display()
    );

    std::fs::write(temp_config, config_content)
        .context("Failed to write hyprpaper config")?;

    // Kill existing hyprpaper
    std::process::Command::new("pkill")
        .arg("-x")
        .arg("hyprpaper")
        .output()
        .ok();

    // Start hyprpaper with new config
    std::process::Command::new("hyprpaper")
        .arg("-c")
        .arg(temp_config)
        .spawn()
        .context("Failed to start hyprpaper")?;

    println!("  ✓ Set wallpaper");
    Ok(())
}

fn run_init() -> Result<()> {
    println!("🚀 Initializing iro...\n");

    let home = dirs::home_dir().context("Failed to get home directory")?;
    let config_dir = dirs::config_dir().context("Failed to get config directory")?;

    // 1. Create directories
    println!("📁 Creating directories...");
    let wallpaper_dir = home.join("Pictures/wallpaper");
    let iro_config = config_dir.join("iro");
    let iro_templates = iro_config.join("templates");
    let hypr_scripts = config_dir.join("hypr/scripts");

    std::fs::create_dir_all(&wallpaper_dir)?;
    std::fs::create_dir_all(&iro_templates)?;
    std::fs::create_dir_all(&hypr_scripts)?;
    println!("  ✓ Created ~/Pictures/wallpaper");
    println!("  ✓ Created ~/.config/iro/templates");
    println!("  ✓ Created ~/.config/hypr/scripts");

    // 2. Copy templates
    println!("\n📋 Installing templates...");
    let template_engine = TemplateEngine::new()?;
    template_engine.create_default_templates()?;
    println!("  ✓ Installed color templates");

    // 3. Copy wallpaper script
    println!("\n🖼️  Installing wallpaper script...");
    let script_path = hypr_scripts.join("random_wallpaper.sh");
    let current_exe = std::env::current_exe()?;
    let project_dir = current_exe.parent().unwrap().parent().unwrap().parent().unwrap();

    // Check if script exists in project
    if let Ok(script_content) = std::fs::read_to_string(config_dir.join("hypr/scripts/random_wallpaper.sh")) {
        println!("  ✓ Wallpaper script already exists");
    } else {
        println!("  ⚠ Please manually copy random_wallpaper.sh to ~/.config/hypr/scripts/");
    }

    // 4. Shell integration
    println!("\n🐚 Shell integration...");
    let shell_rc = if std::env::var("SHELL").unwrap_or_default().contains("zsh") {
        home.join(".zshrc")
    } else {
        home.join(".bashrc")
    };

    let shell_integration = "\n# iro - dynamic color scheme\n[ -f ~/.config/iro/colors.sh ] && source ~/.config/iro/colors.sh\n";

    if shell_rc.exists() {
        let content = std::fs::read_to_string(&shell_rc)?;
        if !content.contains("iro") {
            std::fs::write(&shell_rc, format!("{}{}", content, shell_integration))?;
            println!("  ✓ Added iro to {}", shell_rc.display());
        } else {
            println!("  ✓ iro already in {}", shell_rc.display());
        }
    } else {
        std::fs::write(&shell_rc, shell_integration)?;
        println!("  ✓ Created {} with iro integration", shell_rc.display());
    }

    // 5. Hyprland integration
    println!("\n🖥️  Hyprland integration...");
    let hyprland_conf = config_dir.join("hypr/hyprland.conf");
    if hyprland_conf.exists() {
        let content = std::fs::read_to_string(&hyprland_conf)?;
        if !content.contains("random_wallpaper.sh") {
            println!("  ⚠ Add this line to your hyprland.conf:");
            println!("    exec-once = ~/.config/hypr/scripts/random_wallpaper.sh");
        } else {
            println!("  ✓ Hyprland already configured");
        }
    } else {
        println!("  ⚠ hyprland.conf not found");
    }

    println!("\n✅ iro initialization complete!");
    println!("\n📝 Next steps:");
    println!("  1. Add wallpapers to ~/Pictures/wallpaper/");
    println!("  2. Run: iro --gui");
    println!("  3. Restart your shell or run: source {}", shell_rc.display());

    Ok(())
}
