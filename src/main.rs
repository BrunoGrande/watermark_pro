use eframe::egui;
use image::DynamicImage;
use std::path::PathBuf;

// Import the processor module we just made
mod processor;

fn main() -> eframe::Result<()> {
    // 1. Set up the window options (size, title)
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0]), // Same size as your Python app
        ..Default::default()
    };

    // 2. Launch the app
    eframe::run_native(
        "Watermark Pro (Rust)",
        options,
        Box::new(|_cc| Ok(Box::new(WatermarkApp::default()))),
    )
}

// This struct holds the "State" of the application
// It's just like the "self.variable" lines in your Python __init__
struct WatermarkApp {
    input_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    watermark_path: Option<PathBuf>,
    
    // Sliders
    opacity: f32,
    scale: f32,
    pos_x: f32,
    pos_y: f32,

    // Internal image storage
    preview_image: Option<DynamicImage>,
    watermark_image: Option<DynamicImage>,
    preview_texture: Option<egui::TextureHandle>, 
    
    status_msg: String, // <--- PUT THIS BACK!
}

// 1. Custom methods go here
impl WatermarkApp {
    // Helper to refresh the preview image
    fn refresh_preview(&mut self, ctx: &egui::Context) {
        if let (Some(base), Some(wm)) = (&self.preview_image, &self.watermark_image) {
            
            // 1. Call your processor!
            let final_img = processor::apply_watermark(
                base, 
                wm, 
                self.opacity, 
                self.scale, 
                self.pos_x, 
                self.pos_y
            );

            // 2. Convert to egui ColorImage
            let size = [final_img.width() as usize, final_img.height() as usize];
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                size, 
                final_img.as_bytes()
            );

            // 3. Upload to GPU
            self.preview_texture = Some(ctx.load_texture(
                "preview_tex", 
                color_image, 
                egui::TextureOptions::LINEAR
            ));
        }
    }
}

// This sets the default values when the app starts
impl Default for WatermarkApp {
    fn default() -> Self {
        Self {
            input_path: None,
            output_path: None,
            watermark_path: None,
            opacity: 0.5,
            scale: 0.20,
            pos_x: 0.5,
            pos_y: 0.5,
            preview_image: None,
            watermark_image: None,
            status_msg: "Ready.".to_owned(),
            preview_texture: None, // NEW
        }
    }
}

// This is where we draw the UI (Equivalent to your setup_ui method)
impl eframe::App for WatermarkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- 1. LEFT PANEL (Controls) ---
        egui::SidePanel::left("left_panel")
            .resizable(false)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Settings");
                ui.separator();

                // Section: Files
                ui.label("📂 Files");
                ui.vertical(|ui| {
                    // Input Folder
                    ui.horizontal(|ui| {
                            if ui.button("Select Input...").clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                    // FIX: We .clone() it here so we can still use 'path' below
                                    self.input_path = Some(path.clone()); 
                                    
                                    // Now we can still use 'path' because we only gave away a copy
                                    if let Ok(read_dir) = std::fs::read_dir(&path) {
                                        for entry in read_dir.flatten() {
                                            let p = entry.path();
                                            if p.is_file() {
                                                if let Ok(img) = image::open(&p) {
                                                    self.preview_image = Some(img);
                                                    self.refresh_preview(ctx);
                                                    break; 
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        // Show the current folder name or "None"
                        let text = self.input_path.as_ref()
                            .and_then(|p| p.file_name())
                            .map(|n| n.to_string_lossy())
                            .unwrap_or("None".into());
                        ui.label(format!("In: {}", text));
                    });

                    // Output Folder
                    ui.horizontal(|ui| {
                        if ui.button("Select Output...").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.output_path = Some(path);
                            }
                        }
                        let text = self.output_path.as_ref()
                            .and_then(|p| p.file_name())
                            .map(|n| n.to_string_lossy())
                            .unwrap_or("None".into());
                        ui.label(format!("Out: {}", text));
                    });

                    // Watermark File
                    ui.horizontal(|ui| {
                            if ui.button("Select Watermark...").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Image", &["png", "jpg", "jpeg"])
                                    .pick_file() 
                                {
                                    // FIX: Clone here too
                                    self.watermark_path = Some(path.clone());
                                    
                                    // Now we can use 'path' to open the image
                                    if let Ok(img) = image::open(&path) {
                                        self.watermark_image = Some(img);
                                        self.refresh_preview(ctx);
                                    }
                                }
                            }
                        let text = self.watermark_path.as_ref()
                            .and_then(|p| p.file_name())
                            .map(|n| n.to_string_lossy())
                            .unwrap_or("None".into());
                        ui.label(format!("Wm: {}", text));
                    });
                });
                
                ui.separator();

                // Section: Appearance
                ui.label("🎨 Appearance");
                ui.add(egui::Slider::new(&mut self.opacity, 0.0..=1.0).text("Opacity"));
                ui.add(egui::Slider::new(&mut self.scale, 0.05..=1.0).text("Scale"));

                ui.separator();

                // Section: Position
                ui.label("📍 Position");
                ui.add(egui::Slider::new(&mut self.pos_x, 0.0..=1.0).text("X Axis"));
                ui.add(egui::Slider::new(&mut self.pos_y, 0.0..=1.0).text("Y Axis"));

                ui.separator();
                
                // Run Button (Disabled if paths are missing)
                let can_run = self.input_path.is_some() && self.output_path.is_some() && self.watermark_path.is_some();
                ui.add_enabled_ui(can_run, |ui| {
                    if ui.button("🚀 PROCESS IMAGES").clicked() {
                        self.status_msg = "Processing started...".to_owned();
                        // We will add the batch processing logic later!
                    }
                });
                ui.label(&self.status_msg);
            });

// --- 2. CENTRAL PANEL (Preview) ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Preview");
            ui.centered_and_justified(|ui| {
                // If we have a texture, draw it!
                if let Some(texture) = &self.preview_texture {
                    ui.image(texture);
                } else {
                    ui.label("Load a folder to see the preview.");
                }
            });
        });
    }
}