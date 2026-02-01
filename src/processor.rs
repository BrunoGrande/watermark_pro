use image::{imageops, DynamicImage, GenericImageView};

pub fn apply_watermark(
    base: &DynamicImage,
    watermark: &DynamicImage,
    opacity: f32,
    scale: f32,
    pos_x: f32,
    pos_y: f32,
) -> DynamicImage {
    // --- 1. RESIZE (Equivalent to PIL .resize) ---
    // Calculate new size based on the base image width
    let new_width = (base.width() as f32 * scale) as u32;
    // Maintain aspect ratio
    let new_height = ((new_width as f32 / watermark.width() as f32) * watermark.height() as f32) as u32;
    
    // Resize using Lanczos3 (high quality, same as your Python script)
    let mut wm_resized = watermark.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);

    // --- 2. OPACITY (Equivalent to PIL putalpha) ---
    // In Rust, we iterate over the pixels to adjust alpha if needed
    if opacity < 1.0 {
        // We ensure the image is in RGBA format to access the alpha channel
        if let Some(rgba_img) = wm_resized.as_mut_rgba8() {
            for pixel in rgba_img.pixels_mut() {
                // Scale the current alpha value by the opacity factor
                let new_alpha = (pixel[3] as f32 * opacity) as u8;
                pixel[3] = new_alpha;
            }
        }
    }

    // --- 3. POSITION (Equivalent to calculating x/y) ---
    let max_x = base.width().saturating_sub(wm_resized.width());
    let max_y = base.height().saturating_sub(wm_resized.height());
    
    let x = (max_x as f32 * pos_x) as i64;
    let y = (max_y as f32 * pos_y) as i64;

    // --- 4. OVERLAY (Equivalent to canvas.paste) ---
    let mut final_image = base.clone();
    imageops::overlay(&mut final_image, &wm_resized, x, y);
    
    final_image
}