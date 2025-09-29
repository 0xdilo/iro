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
                .help("Path to wallpaper image (or use --random)")
                .required(false)
                .index(1)
        )
        .arg(
            Arg::new("random")
                .long("random")
                .help("Select a random wallpaper from ~/Pictures/wallpaper/")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("monitors")
                .short('m')
                .long("monitors")
                .value_name("MONITOR1,MONITOR2,...")
                .help("Comma-separated list of monitors (e.g., eDP-1,DP-3,DP-1). If not specified, applies to all monitors")
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
    let random_mode = matches.get_flag("random");
    let monitors = matches.get_one::<String>("monitors");

    // Handle init mode
    if init_mode {
        return run_init();
    }

    // Handle GUI mode
    let wallpaper_path = if gui_mode {
        open_wallpaper_picker()?
    } else if random_mode {
        select_random_wallpaper()?
    } else {
        match matches.get_one::<String>("wallpaper") {
            Some(path) => PathBuf::from(path),
            None => {
                eprintln!("Error: Wallpaper path is required (or use --gui/--random mode)");
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
    
    // Set wallpaper
    let monitor_list = monitors.map(|s| s.as_str());
    set_wallpaper(&wallpaper_path, monitor_list)?;

    // Reload applications
    if gui_mode || should_reload || random_mode {
        println!("🔄 Reloading applications...");
        reload_applications()?;
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

fn set_wallpaper(wallpaper_path: &PathBuf, monitors: Option<&str>) -> Result<()> {
    println!("🖼️  Setting wallpaper...");

    let wallpaper_str = wallpaper_path.to_str()
        .context("Invalid wallpaper path")?;

    // Get list of monitors
    let monitor_list = if let Some(mon_str) = monitors {
        // Use specified monitors
        mon_str.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        // Get all monitors from hyprctl
        get_all_monitors()?
    };

    if monitor_list.is_empty() {
        return Err(anyhow::anyhow!("No monitors found"));
    }

    // Check if hyprpaper is running
    let hyprpaper_running = std::process::Command::new("pgrep")
        .arg("-x")
        .arg("hyprpaper")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !hyprpaper_running {
        // Create a minimal hyprpaper config
        println!("  ⚙️  Starting hyprpaper...");
        let config_dir = dirs::config_dir().context("Failed to get config directory")?;
        let hyprpaper_conf = config_dir.join("hypr/hyprpaper.conf");

        // Create hyprpaper.conf if it doesn't exist
        if !hyprpaper_conf.exists() {
            std::fs::write(&hyprpaper_conf, "# Generated by iro\nsplash = false\n")
                .context("Failed to create hyprpaper.conf")?;
        }

        std::process::Command::new("hyprpaper")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .context("Failed to start hyprpaper")?;

        // Give it a moment to start
        std::thread::sleep(std::time::Duration::from_millis(800));
    }

    // Preload the wallpaper first
    let preload_output = std::process::Command::new("hyprctl")
        .args(["hyprpaper", "preload", wallpaper_str])
        .output()
        .context("Failed to preload wallpaper")?;

    if !preload_output.status.success() {
        let err_msg = String::from_utf8_lossy(&preload_output.stderr);
        eprintln!("  ⚠ Warning: Failed to preload wallpaper: {}", err_msg);
    }

    // Apply wallpaper to each monitor
    for monitor in &monitor_list {
        let output = std::process::Command::new("hyprctl")
            .args(["hyprpaper", "wallpaper", &format!("{},{}", monitor, wallpaper_str)])
            .output()
            .context("Failed to set wallpaper")?;

        if output.status.success() {
            println!("  ✓ Set wallpaper on {}", monitor);
        } else {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            eprintln!("  ⚠ Warning: Failed to set wallpaper on {}: {}", monitor, err_msg);
        }
    }

    Ok(())
}

fn get_all_monitors() -> Result<Vec<String>> {
    let output = std::process::Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .context("Failed to get monitors")?;

    let monitors_json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .context("Failed to parse monitors JSON")?;

    let mut monitors = Vec::new();
    if let Some(array) = monitors_json.as_array() {
        for monitor in array {
            if let Some(name) = monitor.get("name").and_then(|n| n.as_str()) {
                monitors.push(name.to_string());
            }
        }
    }

    Ok(monitors)
}

fn select_random_wallpaper() -> Result<PathBuf> {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    let home = dirs::home_dir().context("Failed to get home directory")?;
    let wallpaper_dir = home.join("Pictures/wallpaper");

    if !wallpaper_dir.exists() {
        return Err(anyhow::anyhow!(
            "Wallpaper directory not found: {}. Run 'iro --init' first.",
            wallpaper_dir.display()
        ));
    }

    let mut wallpapers: Vec<PathBuf> = std::fs::read_dir(&wallpaper_dir)?
        .filter_map(|entry| entry.ok())
        .map(|e| e.path())
        .filter(|path| {
            path.is_file() && matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("jpg") | Some("jpeg") | Some("png") | Some("webp")
            )
        })
        .collect();

    if wallpapers.is_empty() {
        return Err(anyhow::anyhow!(
            "No wallpapers found in {}",
            wallpaper_dir.display()
        ));
    }

    let mut rng = thread_rng();
    let selected = wallpapers.choose(&mut rng).unwrap().clone();

    println!("🎲 Selected random wallpaper: {}", selected.file_name().unwrap().to_string_lossy());

    Ok(selected)
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
