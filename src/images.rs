use image::codecs::bmp::BmpEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::{ColorType, DynamicImage, ImageError};
use image::imageops::FilterType;
use crate::info::{ImageMirroring, ImageMode, ImageRotation};
use crate::Kind;

/// Converts image into image data depending on provided kind of device
pub fn convert_image(kind: Kind, image: DynamicImage) -> Result<Vec<u8>, ImageError> {
    let image_format = kind.key_image_format();

    // Ensuring size of the image
    let (ws, hs) = image_format.size;

    let image = image.resize_exact(ws as u32, hs as u32, FilterType::Nearest);

    // Applying rotation
    let image = match image_format.rotation {
        ImageRotation::Rot0 => image,
        ImageRotation::Rot90 => image.rotate90(),
        ImageRotation::Rot180 => image.rotate180(),
        ImageRotation::Rot270 => image.rotate270()
    };

    // Applying mirroring
    let image = match image_format.mirror {
        ImageMirroring::None => image,
        ImageMirroring::X => image.fliph(),
        ImageMirroring::Y => image.flipv(),
        ImageMirroring::Both => image.fliph().flipv()
    };

    let mut image_data = image.into_rgb8().to_vec();

    // Encoding image
    match image_format.mode {
        ImageMode::None => Ok(vec![]),
        ImageMode::BMP => {
            let mut buf = Vec::new();
            let mut encoder = BmpEncoder::new(&mut buf);
            encoder.encode(&image_data, ws as u32, hs as u32, ColorType::Rgb8)?;
            Ok(buf)
        }
        ImageMode::JPEG => {
            let mut buf = Vec::new();
            let mut encoder = JpegEncoder::new_with_quality(&mut buf, 100);
            encoder.encode(&image_data, ws as u32, hs as u32, ColorType::Rgb8)?;
            Ok(buf)
        }
    }
}