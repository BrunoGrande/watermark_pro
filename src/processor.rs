use image::{imageops, DynamicImage, RgbaImage};
use fast_image_resize as fr;
use fast_image_resize::images::Image;
use fast_image_resize::{ResizeOptions, ResizeAlg, FilterType};

pub fn apply_watermark(
    base: &DynamicImage,
    watermark: &DynamicImage,
    opacity: f32,
    scale: f32,
    pos_x: f32,
    pos_y: f32,
) -> DynamicImage {
    
    // 1. Calculate new dimensions
    let new_width = ((base.width() as f32 * scale) as u32).max(1);
    let new_height = (((new_width as f32 / watermark.width() as f32) * watermark.height() as f32) as u32).max(1);

    // 2. High-Speed Resize
    let wm_width = watermark.width();
    let wm_height = watermark.height();
    
    let mut src_buffer = watermark.to_rgba8(); 

    let src_image = Image::from_slice_u8(
        wm_width, 
        wm_height, 
        src_buffer.as_mut(),
        fr::PixelType::U8x4
    ).unwrap();

    let mut dst_image = Image::new(new_width, new_height, src_image.pixel_type());

    let mut resizer = fr::Resizer::new();
    let options = ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Lanczos3));
    resizer.resize(&src_image, &mut dst_image, &options).unwrap();

    let mut wm_resized = RgbaImage::from_raw(new_width, new_height, dst_image.into_vec()).unwrap();

    // 3. Apply Opacity
    if opacity < 1.0 {
        for pixel in wm_resized.pixels_mut() {
            let new_alpha = (pixel[3] as f32 * opacity) as u8;
            pixel[3] = new_alpha;
        }
    }

    // 4. Calculate Position
    let max_x = base.width().saturating_sub(wm_resized.width());
    let max_y = base.height().saturating_sub(wm_resized.height());
    
    let x = (max_x as f32 * pos_x) as i64;
    let y = (max_y as f32 * pos_y) as i64;

    // 5. Overlay
    let mut final_image = base.clone();
    imageops::overlay(&mut final_image, &wm_resized, x, y);
    
    final_image
}