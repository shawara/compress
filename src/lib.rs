use std::cmp::{max, min};
use wasm_bindgen::prelude::*;
use lopdf::Document;
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::io::Cursor;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub fn compress_pdf(input_data: &[u8], quality: f32) -> Vec<u8> {
    let mut doc = Document::load_mem(input_data).expect("Failed to load PDF");

    for (_, object) in doc.objects.iter_mut() {
        if let lopdf::Object::Stream(ref mut stream) = object {
            if let Ok(filters) = stream.dict.get(b"Subtype") {
                if filters.as_name().map_or(false, |filter| filter == b"Image") {
                    let image_data = stream.content.clone();
                    let image = image::load_from_memory(&image_data).expect("Failed to load image");

                    let ratio = 1.0_f32.max(min(image.width(), image.height()) as f32 / quality);
                    let compressed_image: DynamicImage = image.resize(
                        (image.width() as f32 / ratio) as u32,
                        (image.height() as f32 / ratio) as u32,
                        image::imageops::FilterType::Triangle)
                        .into_rgb8()
                        .into();

                    let mut compressed_data = Vec::new();
                    compressed_image.write_to(&mut Cursor::new(&mut compressed_data), ImageFormat::Jpeg).expect("Failed to write compressed image");
                    stream.content = compressed_data;
                }
            }
        }
    }

    let mut output_data = Vec::new();
    doc.save_to(&mut output_data).expect("Failed to save PDF");
    output_data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
