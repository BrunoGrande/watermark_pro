#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // <-
use eframe::egui;
use image::{DynamicImage, ImageBuffer, Rgb}; // Added ImageBuffer and Rgb for RAW handling
use std::path::{Path, PathBuf};

mod processor;

const ALLOWED_EXTS: &[&str] = &[
    "png", "jpg", "jpeg", "webp",               // Standard
    "nef", "cr2", "arw", "dng", "orf", "rw2", "raf" // RAW Formats
];

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Watermark Pro (Rust)",
        options,
        Box::new(|_cc| Ok(Box::new(WatermarkApp::default()))),
    )
}

struct WatermarkApp {
    input_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    watermark_path: Option<PathBuf>,
    
    is_batch_mode: bool,
    opacity: f32,
    scale: f32,
    pos_x: f32,
    pos_y: f32,

    base_image_cpu: Option<DynamicImage>,
    watermark_image_cpu: Option<DynamicImage>,
    base_texture: Option<egui::TextureHandle>,
    watermark_texture: Option<egui::TextureHandle>,
    
    status_msg: String, 
}

impl Default for WatermarkApp {
    fn default() -> Self {
        Self {
            input_path: None,
            output_path: None,
            watermark_path: None,
            is_batch_mode: false,
            opacity: 0.5,
            scale: 0.20,
            pos_x: 0.5,
            pos_y: 0.5,
            base_image_cpu: None,
            watermark_image_cpu: None,
            base_texture: None,
            watermark_texture: None,
            status_msg: "Ready.".to_owned(),
        }
    }
}

// 2. TRUE RAW LOADER (High Quality 16-bit)
fn load_image(path: &Path) -> Result<DynamicImage, String> {
    let ext = path.extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();

    let raw_exts = ["nef", "cr2", "arw", "dng", "orf", "rw2", "raf"];

    if raw_exts.contains(&ext.as_str()) {
        // Decode the file
        match rawloader::decode_file(path) {
            Ok(raw) => {
                if raw.width == 0 || raw.height == 0 {
                    return Err("Decoded RAW has zero dimensions".to_owned());
                }

                // UNWRAP THE ENUM: Check if it's Integer or Float data
                match raw.data {
                    rawloader::RawImageData::Integer(data) => {
                        // Create 16-bit Image Buffer
                        if let Some(buffer) = ImageBuffer::<Rgb<u16>, Vec<u16>>::from_raw(
                            raw.width as u32, 
                            raw.height as u32, 
                            data
                        ) {
                            Ok(DynamicImage::ImageRgb16(buffer))
                        } else {
                            Err("Failed to create image buffer from RAW data".to_owned())
                        }
                    }
                    rawloader::RawImageData::Float(_) => {
                        Err("Float RAW data is not currently supported.".to_owned())
                    }
                }
            },
            Err(e) => Err(format!("Failed to decode RAW: {}", e)),
        }
    } else {
        // Standard Image
        image::open(path).map_err(|e| e.to_string())
    }
}

impl WatermarkApp {
    fn load_texture(&self, ctx: &egui::Context, img: &DynamicImage, name: &str) -> egui::TextureHandle {
        let img_buffer = img.to_rgba8();
        let size = [img_buffer.width() as usize, img_buffer.height() as usize];
        let pixels = img_buffer.as_flat_samples();
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
        ctx.load_texture(name, color_image, egui::TextureOptions::LINEAR)
    }

    fn run_processing(&mut self) {
        if let (Some(out_dir), Some(wm)) = (&self.output_path, &self.watermark_image_cpu) {
            self.status_msg = "Processing...".to_owned();
            
            let mut files_to_process = Vec::new();

            if self.is_batch_mode {
                if let Some(in_dir) = &self.input_path {
                    for entry in walkdir::WalkDir::new(in_dir).into_iter().filter_map(|e| e.ok()) {
                        let p = entry.path();
                        let ext = p.extension().unwrap_or_default().to_string_lossy().to_lowercase();
                        if p.is_file() && ALLOWED_EXTS.contains(&ext.as_str()) {
                            files_to_process.push(p.to_owned());
                        }
                    }
                }
            } else if let Some(in_file) = &self.input_path {
                files_to_process.push(in_file.clone());
            }

            let count = files_to_process.len();
            for file_path in files_to_process.iter() {
                if let Ok(img) = load_image(file_path) {
                    
                    let final_img = processor::apply_watermark(
                        &img, wm, self.opacity, self.scale, self.pos_x, self.pos_y
                    );

                    let file_name = file_path.file_stem().unwrap_or_default();
                    // Save as PNG
                    let save_name = format!("{}.png", file_name.to_string_lossy());
                    let save_path = out_dir.join(save_name);
                    
                    let _ = final_img.save(save_path); 
                    println!("Saved: {:?}", file_name);
                } else {
                    println!("Skipped (Load Error): {:?}", file_path);
                }
            }
            self.status_msg = format!("Done! Processed {} images.", count);
        }
    }
}

impl eframe::App for WatermarkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        // --- DRAG AND DROP ---
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            let dropped = ctx.input(|i| i.raw.dropped_files.clone());
            if let Some(file) = dropped.first() {
                if let Some(path) = &file.path {
                    if path.is_dir() {
                        self.is_batch_mode = true;
                        self.input_path = Some(path.clone());
                        
                        let preview_file = walkdir::WalkDir::new(path).into_iter()
                            .filter_map(|e| e.ok())
                            .find(|e| {
                                let ext = e.path().extension().unwrap_or_default().to_string_lossy().to_lowercase();
                                e.path().is_file() && ALLOWED_EXTS.contains(&ext.as_str())
                            })
                            .map(|e| e.path().to_owned());

                        if let Some(f) = preview_file {
                            if let Ok(img) = load_image(&f) {
                                self.base_image_cpu = Some(img.clone());
                                self.base_texture = Some(self.load_texture(ctx, &img, "base_tex"));
                            }
                        }
                    } else {
                        self.is_batch_mode = false;
                        self.input_path = Some(path.clone());
                        if let Ok(img) = load_image(path) {
                            self.base_image_cpu = Some(img.clone());
                            self.base_texture = Some(self.load_texture(ctx, &img, "base_tex"));
                        }
                    }
                }
            }
        }

        egui::SidePanel::left("left_panel")
            .resizable(false)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Settings");
                ui.separator();

                ui.label("📂 Files");
                ui.vertical(|ui| {
                    ui.checkbox(&mut self.is_batch_mode, "Batch Mode (Folder)");
                    ui.add_space(5.0);

                    // 1. SELECT INPUT
                    ui.horizontal(|ui| {
                        let btn_text = if self.is_batch_mode { "Select Input Folder" } else { "Select Input Image" };
                        if ui.button(btn_text).clicked() {
                            let path_opt = if self.is_batch_mode {
                                rfd::FileDialog::new().pick_folder()
                            } else {
                                rfd::FileDialog::new().add_filter("Image", ALLOWED_EXTS).pick_file()
                            };

                            if let Some(path) = path_opt {
                                self.input_path = Some(path.clone());
                                
                                let preview_file = if self.is_batch_mode {
                                    walkdir::WalkDir::new(&path).into_iter()
                                        .filter_map(|e| e.ok())
                                        .find(|e| {
                                            let ext = e.path().extension().unwrap_or_default().to_string_lossy().to_lowercase();
                                            e.path().is_file() && ALLOWED_EXTS.contains(&ext.as_str())
                                        })
                                        .map(|e| e.path().to_owned())
                                } else {
                                    Some(path)
                                };

                                if let Some(f) = preview_file {
                                    if let Ok(img) = load_image(&f) {
                                        self.base_image_cpu = Some(img.clone());
                                        self.base_texture = Some(self.load_texture(ctx, &img, "base_tex"));
                                    }
                                }
                            }
                        }
                        let text = self.input_path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy()).unwrap_or("None".into());
                        ui.label(format!("In: {}", text));
                    });

                    // 2. SELECT OUTPUT
                    ui.horizontal(|ui| {
                        if ui.button("Select Output Folder").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.output_path = Some(path);
                            }
                        }
                        let text = self.output_path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy()).unwrap_or("None".into());
                        ui.label(format!("Out: {}", text));
                    });

                    // 3. SELECT WATERMARK
                    ui.horizontal(|ui| {
                        if ui.button("Select Watermark").clicked() {
                            if let Some(path) = rfd::FileDialog::new().add_filter("Image", &["png", "jpg", "jpeg", "webp"]).pick_file() {
                                self.watermark_path = Some(path.clone());
                                if let Ok(img) = image::open(&path) {
                                    self.watermark_image_cpu = Some(img.clone());
                                    self.watermark_texture = Some(self.load_texture(ctx, &img, "wm_tex"));
                                }
                            }
                        }
                        let text = self.watermark_path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy()).unwrap_or("None".into());
                        ui.label(format!("Wm: {}", text));
                    });
                });
                
                ui.separator();
                ui.label("🎨 Appearance");
                ui.add(egui::Slider::new(&mut self.opacity, 0.0..=1.0).text("Opacity"));
                ui.add(egui::Slider::new(&mut self.scale, 0.05..=1.0).text("Scale"));

                ui.separator();
                ui.label("📍 Position");
                ui.add(egui::Slider::new(&mut self.pos_x, 0.0..=1.0).text("X Axis"));
                ui.add(egui::Slider::new(&mut self.pos_y, 0.0..=1.0).text("Y Axis"));

                ui.separator();
                let can_run = self.input_path.is_some() && self.output_path.is_some() && self.watermark_image_cpu.is_some();
                ui.add_enabled_ui(can_run, |ui| {
                    if ui.button("🚀 PROCESS IMAGES").clicked() {
                        self.run_processing(); 
                    }
                });
                ui.label(&self.status_msg);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Preview");
            if let Some(base_tex) = &self.base_texture {
                let available_size = ui.available_size();
                let (response, painter) = ui.allocate_painter(available_size, egui::Sense::hover());

                let base_w = base_tex.size()[0] as f32;
                let base_h = base_tex.size()[1] as f32;
                let image_aspect = base_w / base_h;
                let view_aspect = available_size.x / available_size.y;

                let (draw_w, draw_h) = if view_aspect > image_aspect {
                    (available_size.y * image_aspect, available_size.y)
                } else {
                    (available_size.x, available_size.x / image_aspect)
                };

                let draw_rect = egui::Rect::from_center_size(response.rect.center(), egui::vec2(draw_w, draw_h));

                painter.image(
                    base_tex.id(),
                    draw_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                if let Some(wm_tex) = &self.watermark_texture {
                    let wm_aspect = wm_tex.size()[0] as f32 / wm_tex.size()[1] as f32;
                    let wm_display_w = draw_rect.width() * self.scale;
                    let wm_display_h = wm_display_w / wm_aspect;

                    let range_x = draw_rect.width() - wm_display_w;
                    let range_y = draw_rect.height() - wm_display_h;
                    let offset_x = range_x * self.pos_x;
                    let offset_y = range_y * self.pos_y;

                    let wm_rect = egui::Rect::from_min_size(
                        draw_rect.min + egui::vec2(offset_x, offset_y),
                        egui::vec2(wm_display_w, wm_display_h)
                    );

                    let alpha_byte = (self.opacity * 255.0) as u8;
                    let tint = egui::Color32::from_white_alpha(alpha_byte);

                    painter.image(
                        wm_tex.id(),
                        wm_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        tint,
                    );
                }
            } else {
                ui.centered_and_justified(|ui| ui.label("Drag & Drop a RAW/Image folder here!"));
            }
        });
    }
}