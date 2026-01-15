use anyhow::{Context, Result};
use clap::{Arg, Command};
use std::path::PathBuf;

mod color_extractor;
mod template_engine;
mod config_generator;
mod gui;
mod config;
mod palette;

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
            Arg::new("wallpapers")
                .help("Wallpaper image(s) - provide one per monitor or single for all")
                .required(false)
                .num_args(0..)
        )
        .arg(
            Arg::new("random")
                .long("random")
                .help("Select a single random wallpaper for all monitors")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("random-each")
                .long("random-each")
                .help("Select different random wallpaper for each monitor")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("primary")
                .short('p')
                .long("primary")
                .value_name("INDEX")
                .help("Index of wallpaper to use for theme extraction (0-based, default: 0)")
                .value_parser(clap::value_parser!(usize))
        )
        .arg(
            Arg::new("monitors")
                .short('m')
                .long("monitors")
                .value_name("MONITOR1,MONITOR2,...")
                .help("Comma-separated list of monitors (e.g., eDP-1,DP-3). If not specified, uses all monitors")
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
    let random_each_mode = matches.get_flag("random-each");
    let primary_index = matches.get_one::<usize>("primary").copied().unwrap_or(0);
    let monitors = matches.get_one::<String>("monitors");

    // Handle init mode
    if init_mode {
        return run_init();
    }

    // Handle GUI mode
    if gui_mode {
        open_wallpaper_picker()?;
    }

    // Get wallpapers for each monitor
    let (wallpaper_paths, primary_wallpaper) = if random_mode {
        // --random: same random wallpaper on all screens
        let wp = select_random_wallpaper()?;
        (vec![wp.clone()], wp)
    } else if random_each_mode {
        // --random-each: different random wallpaper per screen
        get_random_wallpapers_per_monitor(monitors, primary_index)?
    } else {
        // Manual mode: specify wallpapers, use --primary for theme
        let wallpapers: Vec<&str> = matches
            .get_many::<String>("wallpapers")
            .map(|vals| vals.map(|s| s.as_str()).collect())
            .unwrap_or_default();

        if wallpapers.is_empty() {
            anyhow::bail!("Error: Wallpaper path(s) required (or use --gui/--random/--random-each)");
        }

        let paths: Vec<PathBuf> = wallpapers.iter().map(PathBuf::from).collect();
        let primary = paths.get(primary_index).unwrap_or(&paths[0]).clone();
        (paths, primary)
    };

    println!("🎨 iro - Generating color scheme from: {}", primary_wallpaper.display());

    // Extract colors from primary wallpaper
    let extractor = ColorExtractor::new()?;
    let color_scheme = extractor.extract_colors(&primary_wallpaper, theme)?;
    
    println!("✨ Extracted color scheme:");
    print_color_scheme(&color_scheme);
    
    // Generate configurations
    let config_gen = ConfigGenerator::new()?;
    config_gen.generate_configs(&color_scheme)?;
    
    // Set wallpapers
    set_wallpapers(&wallpaper_paths, monitors)?;

    // Reload applications
    if gui_mode || should_reload || random_mode || random_each_mode {
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
    println!("  Colors: {:?}", &scheme.colors[..8]);
}

fn reload_applications() -> Result<()> {
    std::process::Command::new("hyprctl")
        .args(["reload"])
        .output()
        .context("Failed to reload hyprland")?;

    println!("  ✓ Reloaded Hyprland");
    Ok(())
}

fn open_wallpaper_picker() -> Result<PathBuf> {
    // Launch the Rust GUI
    println!("🎨 Launching iro GUI viewer...");
    gui::launch_gui()?;
    
    // GUI handles everything, so we can exit
    std::process::exit(0);
}

fn set_wallpapers(wallpaper_paths: &[PathBuf], monitors: Option<&String>) -> Result<()> {
    println!("🖼️  Setting wallpaper(s)...");

    // Get list of monitors
    let monitor_list = if let Some(mon_str) = monitors {
        mon_str.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        get_all_monitors()?
    };

    if monitor_list.is_empty() {
        return Err(anyhow::anyhow!("No monitors found"));
    }

    let config_dir = dirs::config_dir().context("Failed to get config directory")?;
    let hyprpaper_conf = config_dir.join("hypr/hyprpaper.conf");

    // Build hyprpaper config content
    let mut config_content = String::from("# Generated by iro\nsplash = false\nipc = on\n\n");

    // Preload all unique wallpapers
    for wallpaper_path in wallpaper_paths {
        let wallpaper_str = wallpaper_path.to_str().context("Invalid wallpaper path")?;
        config_content.push_str(&format!("preload = {}\n", wallpaper_str));
    }
    config_content.push('\n');

    // Assign wallpapers to monitors
    for (i, monitor) in monitor_list.iter().enumerate() {
        let wallpaper_idx = i.min(wallpaper_paths.len() - 1);
        let wallpaper_path = &wallpaper_paths[wallpaper_idx];
        let wallpaper_str = wallpaper_path.to_str().unwrap();
        config_content.push_str(&format!("wallpaper = {},{}\n", monitor, wallpaper_str));
    }

    // Write config (for persistence on restart)
    std::fs::write(&hyprpaper_conf, &config_content)
        .context("Failed to write hyprpaper.conf")?;

    // Check if hyprpaper is running
    let hyprpaper_running = std::process::Command::new("pgrep")
        .args(["-x", "hyprpaper"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !hyprpaper_running {
        // Start hyprpaper if not running
        std::process::Command::new("hyprpaper")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .context("Failed to start hyprpaper")?;
        std::thread::sleep(std::time::Duration::from_millis(400));
    }

    // Set wallpapers via IPC (fast, no restart needed)
    for (i, monitor) in monitor_list.iter().enumerate() {
        let wallpaper_idx = i.min(wallpaper_paths.len() - 1);
        let wallpaper_path = &wallpaper_paths[wallpaper_idx];
        let wallpaper_str = wallpaper_path.to_str().unwrap();

        let _ = std::process::Command::new("hyprctl")
            .args(["hyprpaper", "wallpaper", &format!("{},{}", monitor, wallpaper_str)])
            .output();

        println!("  ✓ Set {} on {}", wallpaper_path.file_name().unwrap().to_string_lossy(), monitor);
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

    Ok(monitors_json
        .as_array()
        .map(|array| {
            array
                .iter()
                .filter_map(|monitor| monitor.get("name")?.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default())
}

fn get_wallpapers_list() -> Result<Vec<PathBuf>> {
    let config = config::IroConfig::load().unwrap_or_default();
    let wallpaper_dir = config.wallpaper_path();

    if !wallpaper_dir.exists() {
        anyhow::bail!(
            "Wallpaper directory not found: {}. Run 'iro --init' first or set wallpaper_dir in ~/.config/iro/config.toml",
            wallpaper_dir.display()
        );
    }

    let wallpapers: Vec<PathBuf> = std::fs::read_dir(&wallpaper_dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|path| {
            path.is_file() && matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("jpg" | "jpeg" | "png" | "webp")
            )
        })
        .collect();

    if wallpapers.is_empty() {
        anyhow::bail!("No wallpapers found in {}", wallpaper_dir.display());
    }

    Ok(wallpapers)
}

fn select_random_wallpaper() -> Result<PathBuf> {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    let wallpapers = get_wallpapers_list()?;
    let mut rng = thread_rng();
    let selected = wallpapers.choose(&mut rng).unwrap().clone();

    println!("🎲 Selected random wallpaper: {}", selected.file_name().unwrap().to_string_lossy());

    Ok(selected)
}

fn get_random_wallpapers_per_monitor(monitors: Option<&String>, primary_index: usize) -> Result<(Vec<PathBuf>, PathBuf)> {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    let wallpapers = get_wallpapers_list()?;
    let mut rng = thread_rng();

    // Get monitor list
    let monitor_list = if let Some(mon_str) = monitors {
        mon_str.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        get_all_monitors()?
    };

    if monitor_list.is_empty() {
        anyhow::bail!("No monitors found");
    }

    // Select random wallpapers for each monitor
    let mut selected_wallpapers = Vec::with_capacity(monitor_list.len());
    let mut available_wallpapers = wallpapers;

    println!("🎲 Selecting random wallpaper for each monitor:");
    for monitor in &monitor_list {
        if let Some(selected) = available_wallpapers.choose(&mut rng).cloned() {
            println!("  {} → {}", monitor, selected.file_name().unwrap().to_string_lossy());
            selected_wallpapers.push(selected.clone());

            // Remove selected to avoid duplicates if possible
            if available_wallpapers.len() > 1 {
                available_wallpapers.retain(|p| p != &selected);
            }
        }
    }

    // Get primary wallpaper for theme extraction
    let primary_wallpaper = selected_wallpapers
        .get(primary_index)
        .or_else(|| selected_wallpapers.first())
        .context("No wallpapers selected")?
        .clone();

    Ok((selected_wallpapers, primary_wallpaper))
}

fn run_init() -> Result<()> {
    println!("🚀 Initializing iro...\n");

    let home = dirs::home_dir().context("Failed to get home directory")?;
    let config_dir = dirs::config_dir().context("Failed to get config directory")?;

    // Load or create config
    let iro_cfg = config::IroConfig::load().unwrap_or_default();
    let wallpaper_dir = iro_cfg.wallpaper_path();

    // 1. Create directories
    println!("📁 Creating directories...");
    let iro_config = config_dir.join("iro");
    let iro_templates = iro_config.join("templates");

    std::fs::create_dir_all(&wallpaper_dir)?;
    std::fs::create_dir_all(&iro_templates)?;
    println!("  ✓ Created {}", wallpaper_dir.display());
    println!("  ✓ Created ~/.config/iro/templates");

    // 2. Copy templates
    println!("\n📋 Installing templates...");
    let template_engine = TemplateEngine::new()?;
    template_engine.create_default_templates()?;
    println!("  ✓ Installed color templates");

    // 3. Shell integration
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

    println!("\n✅ iro initialization complete!");
    println!("\n📝 Next steps:");
    println!("  1. Add wallpapers to {}", wallpaper_dir.display());
    println!("  2. Run: iro --gui");
    println!("  3. Restart your shell or run: source {}", shell_rc.display());
    println!("\n💡 Optional: Add to your hyprland.conf for automatic wallpaper on startup:");
    println!("    exec-once = iro --random");
    println!("\n⚙️  Config: ~/.config/iro/config.toml");
    println!("    wallpaper_dir = \"{}\"", iro_cfg.wallpaper_dir);

    Ok(())
}
