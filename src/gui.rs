use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use anyhow::{Context, Result};
use eframe::egui;
use crate::{ColorExtractor, ConfigGenerator};

pub struct WallpaperPickerApp {
    wallpaper_dir: PathBuf,
    wallpapers: Vec<PathBuf>,
    selected_index: Option<usize>,
    thumbnails: Arc<Mutex<Vec<Option<egui::ColorImage>>>>,
    texture_cache: Vec<Option<egui::TextureHandle>>,
    status_message: String,
    applying_theme: bool,
    theme_sender: Option<mpsc::Sender<PathBuf>>,
    theme_receiver: mpsc::Receiver<String>,
    thumbnail_receiver: mpsc::Receiver<(usize, egui::ColorImage)>,
    search_filter: String,
    grid_columns: usize,
    loading_started: bool,
}

impl WallpaperPickerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let wallpaper_dir = dirs::home_dir()
            .map(|h| h.join("Pictures").join("wallpaper"))
            .unwrap_or_else(|| PathBuf::from("."));

        let (theme_sender, theme_receiver_internal) = mpsc::channel::<PathBuf>();
        let (status_sender, theme_receiver) = mpsc::channel::<String>();
        let (thumbnail_sender, thumbnail_receiver) = mpsc::channel::<(usize, egui::ColorImage)>();
        let thumbnail_loader = thumbnail_sender.clone();

        // Spawn background thread for applying themes
        thread::spawn(move || {
            while let Ok(wallpaper_path) = theme_receiver_internal.recv() {
                let result = apply_theme_background(&wallpaper_path);
                let message = match result {
                    Ok(_) => "✅ Theme applied successfully!".to_string(),
                    Err(e) => format!("❌ Error: {}", e),
                };
                let _ = status_sender.send(message);
            }
        });

        let mut app = Self {
            wallpaper_dir,
            wallpapers: Vec::new(),
            selected_index: None,
            thumbnails: Arc::new(Mutex::new(Vec::new())),
            texture_cache: Vec::new(),
            status_message: "Loading wallpapers...".to_string(),
            applying_theme: false,
            theme_sender: Some(theme_sender),
            theme_receiver,
            thumbnail_receiver,
            search_filter: String::new(),
            grid_columns: 4,
            loading_started: false,
        };

        app.load_wallpapers();

        // Start loading thumbnails immediately with the sender
        if !app.wallpapers.is_empty() {
            app.start_loading_thumbnails(thumbnail_loader);
        }

        app
    }

    fn load_wallpapers(&mut self) {
        self.wallpapers.clear();
        if let Ok(mut thumbnails) = self.thumbnails.lock() {
            thumbnails.clear();
        }
        self.texture_cache.clear();

        if self.wallpaper_dir.exists() {
            let extensions = ["jpg", "jpeg", "png", "webp", "gif", "bmp", "tiff"];

            if let Ok(entries) = std::fs::read_dir(&self.wallpaper_dir) {
                for entry in entries.flatten() {
                    if let Some(ext) = entry.path().extension() {
                        if let Some(ext_str) = ext.to_str() {
                            if extensions.contains(&ext_str.to_lowercase().as_str()) {
                                self.wallpapers.push(entry.path());
                            }
                        }
                    }
                }
            }
        }

        self.wallpapers.sort();
        if let Ok(mut thumbnails) = self.thumbnails.lock() {
            *thumbnails = vec![None; self.wallpapers.len()];
        }
        self.texture_cache = vec![None; self.wallpapers.len()];
        self.loading_started = false;

        if !self.wallpapers.is_empty() {
            self.selected_index = Some(0);
            self.status_message = format!("{} wallpapers", self.wallpapers.len());
        } else {
            self.status_message = format!("no wallpapers in {}", self.wallpaper_dir.display());
        }
    }

    fn start_loading_thumbnails(&mut self, sender: mpsc::Sender<(usize, egui::ColorImage)>) {
        if self.loading_started {
            return;
        }
        self.loading_started = true;

        let wallpapers = self.wallpapers.clone();

        // Spawn worker threads for parallel loading
        for chunk_idx in 0..4 {
            let wallpapers = wallpapers.clone();
            let sender = sender.clone();

            thread::spawn(move || {
                let chunk_size = (wallpapers.len() + 3) / 4;
                let start = chunk_idx * chunk_size;
                let end = (start + chunk_size).min(wallpapers.len());

                for idx in start..end {
                    if let Some(path) = wallpapers.get(idx) {
                        if let Ok(img) = image::open(path) {
                            // Fast thumbnail - use Triangle filter
                            let thumb = img.resize(180, 120, image::imageops::FilterType::Triangle);
                            let rgba = thumb.to_rgba8();
                            let size = [rgba.width() as usize, rgba.height() as usize];
                            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                            let _ = sender.send((idx, color_image));
                        }
                    }
                }
            });
        }
    }

    fn apply_current_theme(&mut self) {
        if let Some(index) = self.selected_index {
            if self.applying_theme || index >= self.wallpapers.len() {
                return;
            }

            let wallpaper_path = self.wallpapers[index].clone();

            if let Some(sender) = &self.theme_sender {
                if sender.send(wallpaper_path).is_ok() {
                    self.applying_theme = true;
                    self.status_message = "⏳ Applying theme...".to_string();
                }
            }
        }
    }

    fn filtered_wallpapers(&self) -> Vec<(usize, &PathBuf)> {
        self.wallpapers.iter().enumerate()
            .filter(|(_, path)| {
                if self.search_filter.is_empty() {
                    return true;
                }
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.to_lowercase().contains(&self.search_filter.to_lowercase()))
                    .unwrap_or(false)
            })
            .collect()
    }
}

impl eframe::App for WallpaperPickerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for status updates from background thread
        if let Ok(message) = self.theme_receiver.try_recv() {
            self.status_message = message;
            self.applying_theme = false;
        }

        // Receive loaded thumbnails
        while let Ok((idx, color_image)) = self.thumbnail_receiver.try_recv() {
            if idx < self.texture_cache.len() && self.texture_cache[idx].is_none() {
                let texture = ctx.load_texture(
                    format!("thumb_{}", idx),
                    color_image,
                    egui::TextureOptions::default()
                );
                self.texture_cache[idx] = Some(texture);
                ctx.request_repaint();
            }
        }

        // Top panel with minimalist design
        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::none()
                .fill(egui::Color32::from_rgb(15, 15, 20))
                .inner_margin(egui::Margin::symmetric(16.0, 12.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Minimalist title
                    ui.label(egui::RichText::new("iro").size(16.0).color(egui::Color32::from_rgb(160, 160, 170)));

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Simple search
                    let search_response = ui.add(
                        egui::TextEdit::singleline(&mut self.search_filter)
                            .hint_text("search...")
                            .desired_width(160.0)
                            .frame(false)
                    );

                    if !self.search_filter.is_empty() {
                        ui.label(egui::RichText::new("×").size(16.0).color(egui::Color32::from_rgb(140, 140, 150)))
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                            .then(|| self.search_filter.clear());
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Minimalist apply button
                        let apply_text = if self.applying_theme { "applying..." } else { "apply" };
                        let apply_color = if self.applying_theme {
                            egui::Color32::from_rgb(80, 80, 90)
                        } else {
                            egui::Color32::from_rgb(100, 120, 140)
                        };

                        let apply_btn = egui::Button::new(egui::RichText::new(apply_text).size(13.0).color(egui::Color32::WHITE))
                            .fill(apply_color)
                            .rounding(4.0)
                            .frame(true);

                        if ui.add_enabled(!self.applying_theme && self.selected_index.is_some(), apply_btn).clicked() {
                            self.apply_current_theme();
                        }

                        ui.add_space(8.0);

                        // Grid controls
                        ui.label(egui::RichText::new(&format!("{}×", self.grid_columns)).size(12.0).color(egui::Color32::from_rgb(140, 140, 150)));

                        if ui.button(egui::RichText::new("−").size(14.0)).clicked() && self.grid_columns > 2 {
                            self.grid_columns -= 1;
                        }
                        if ui.button(egui::RichText::new("+").size(14.0)).clicked() && self.grid_columns < 8 {
                            self.grid_columns += 1;
                        }
                    });
                });
            });

        // Bottom status bar
        egui::TopBottomPanel::bottom("bottom_panel")
            .frame(egui::Frame::none()
                .fill(egui::Color32::from_rgb(15, 15, 20))
                .inner_margin(egui::Margin::symmetric(16.0, 10.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&self.status_message).size(11.0).color(egui::Color32::from_rgb(140, 140, 150)));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(index) = self.selected_index {
                            if let Some(path) = self.wallpapers.get(index) {
                                if let Some(filename) = path.file_name() {
                                    ui.label(egui::RichText::new(filename.to_string_lossy()).size(11.0).color(egui::Color32::from_rgb(120, 120, 130)));
                                }
                            }
                        }
                    });
                });
            });

        // Central panel with grid
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(18, 18, 24)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_space(20.0);

                        let filtered: Vec<(usize, PathBuf)> = self.filtered_wallpapers()
                            .into_iter()
                            .map(|(idx, path)| (idx, path.clone()))
                            .collect();

                        if filtered.is_empty() {
                            let wallpaper_dir = self.wallpaper_dir.clone();
                            ui.vertical_centered(|ui| {
                                ui.add_space(120.0);
                                ui.label(egui::RichText::new("no wallpapers").size(14.0).color(egui::Color32::from_rgb(100, 100, 110)));
                                ui.add_space(8.0);
                                ui.label(egui::RichText::new(format!("{}", wallpaper_dir.display())).size(11.0).color(egui::Color32::from_rgb(80, 80, 90)));
                            });
                            return;
                        }

                        // Grid layout
                        let available_width = ui.available_width() - 40.0;
                        let cell_size = (available_width / self.grid_columns as f32).min(260.0);
                        let spacing = 12.0;

                        ui.add_space(10.0);

                        // Use columns for proper grid
                        egui::Grid::new("wallpaper_grid")
                            .spacing([spacing, spacing])
                            .min_col_width(cell_size)
                            .max_col_width(cell_size)
                            .show(ui, |ui| {
                                let mut col = 0;

                                for (real_index, _path) in filtered.iter() {
                                    let is_selected = self.selected_index == Some(*real_index);

                                    let border_color = if is_selected {
                                    egui::Color32::from_rgb(110, 130, 150)
                                } else {
                                    egui::Color32::from_rgb(35, 35, 42)
                                };

                                let frame = egui::Frame::none()
                                    .fill(egui::Color32::from_rgb(25, 25, 32))
                                    .stroke(egui::Stroke::new(1.0, border_color))
                                    .rounding(4.0)
                                    .inner_margin(6.0);

                                frame.show(ui, |ui| {
                                    ui.set_width(cell_size - 30.0);
                                    ui.set_height(cell_size * 0.75);

                                    let response = ui.interact(
                                        ui.available_rect_before_wrap(),
                                        egui::Id::new(format!("wallpaper_{}", real_index)),
                                        egui::Sense::click()
                                    );

                                    if response.clicked() {
                                        self.selected_index = Some(*real_index);
                                    }

                                    if response.double_clicked() {
                                        self.selected_index = Some(*real_index);
                                        self.apply_current_theme();
                                    }

                                    // Draw thumbnail
                                    if let Some(Some(texture)) = self.texture_cache.get(*real_index) {
                                        let img_size = texture.size_vec2();
                                        let scale = ((cell_size - 40.0) / img_size.x).min((cell_size * 0.6) / img_size.y);
                                        let display_size = img_size * scale;

                                        let image_rect = egui::Rect::from_center_size(
                                            ui.available_rect_before_wrap().center(),
                                            display_size
                                        );

                                        ui.put(image_rect, egui::Image::new(texture).fit_to_exact_size(display_size));

                                        // Subtle hover effect
                                        if response.hovered() {
                                            ui.painter().rect_stroke(
                                                image_rect.expand(2.0),
                                                2.0,
                                                egui::Stroke::new(1.0, egui::Color32::from_rgb(110, 130, 150))
                                            );
                                        }
                                    } else {
                                        // Show loading placeholder
                                        ui.centered_and_justified(|ui| {
                                            ui.label(egui::RichText::new("·").size(18.0).color(egui::Color32::from_rgb(50, 50, 60)));
                                        });
                                    }
                                });

                                col += 1;
                                if col >= self.grid_columns {
                                    ui.end_row();
                                    col = 0;
                                }
                            }
                        });

                        ui.add_space(20.0);
                    });
            });

        // Keyboard shortcuts
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Enter) {
                self.apply_current_theme();
            }
            if i.key_pressed(egui::Key::Escape) {
                self.search_filter.clear();
            }
        });

        // Request repaint for animations and ongoing thumbnail loading
        if self.applying_theme || self.texture_cache.iter().any(|t| t.is_none()) {
            ctx.request_repaint();
        }
    }
}

fn apply_theme_background(wallpaper_path: &Path) -> Result<()> {
    // Extract colors
    let extractor = ColorExtractor::new();
    let color_scheme = extractor.extract_colors(&wallpaper_path.to_path_buf(), "dark")?;

    // Generate configs
    let config_gen = ConfigGenerator::new()?;
    config_gen.generate_configs(&color_scheme)?;

    // Reload applications
    reload_applications()?;

    // Set wallpaper
    set_wallpaper_background(wallpaper_path)?;

    Ok(())
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

    Ok(())
}

fn set_wallpaper_background(wallpaper_path: &Path) -> Result<()> {
    // Create hyprpaper config
    let temp_config = "/tmp/iro_gui_hyprpaper.conf";
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

    Ok(())
}

pub fn launch_gui() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("iro - Wallpaper Theme Picker"),
        ..Default::default()
    };

    eframe::run_native(
        "iro",
        options,
        Box::new(|cc| Ok(Box::new(WallpaperPickerApp::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {}", e))?;

    Ok(())
}