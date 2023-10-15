#[allow(unused_imports)]
use std::sync::Arc;
use image::{ColorType, DynamicImage, GenericImageView, ImageBuffer, ImageError};
use image::codecs::bmp::BmpEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;

use crate::{Kind, StreamDeckError};
use crate::info::{ImageMirroring, ImageMode, ImageRotation};

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

    let image_data = image.into_rgb8().to_vec();

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
            let mut encoder = JpegEncoder::new_with_quality(&mut buf, 90);
            encoder.encode(&image_data, ws as u32, hs as u32, ColorType::Rgb8)?;
            Ok(buf)
        }
    }
}

/// Generates a blank button image
pub fn generate_blank_image(kind: Kind) -> Result<Vec<u8>, ImageError> {
    let (width, height) = kind.key_image_format().size;
    let image = ImageBuffer::from_fn(width as u32, height as u32, |_x, _y| {
        image::Luma([0u8])
    });
    Ok(convert_image(kind, image.into())?)
}

/// Converts image into image data depending on provided kind of device, can be safely ran inside [multi_thread](tokio::runtime::Builder::new_multi_thread) runtime
#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub fn convert_image_async(kind: Kind, image: DynamicImage) -> Result<Vec<u8>, crate::StreamDeckError> {
    Ok(tokio::task::block_in_place(move || convert_image(kind, image))?)
}

/// Rect to be used when trying to send image to lcd screen
pub struct ImageRect {
    /// Width of the image
    pub w: u16,

    /// Height of the image
    pub h: u16,

    /// Data of the image row by row as RGB
    pub data: Vec<u8>
}

impl ImageRect {
    /// Converts image to image rect
    pub fn from_image(image: DynamicImage) -> Result<ImageRect, StreamDeckError> {
        let (image_w, image_h) = image.dimensions();

        let image_data = image.into_rgb8().to_vec();

        let mut buf = Vec::new();
        let mut encoder = JpegEncoder::new_with_quality(&mut buf, 90);
        encoder.encode(&image_data, image_w, image_h, ColorType::Rgb8)?;

        Ok(ImageRect {
            w: image_w as u16,
            h: image_h as u16,
            data: buf,
        })
    }

    /// Converts image to image rect, can be safely ran inside [multi_thread](tokio::runtime::Builder::new_multi_thread) runtime
    #[cfg(feature = "async")]
    #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
    pub fn from_image_async(image: DynamicImage) -> Result<ImageRect, StreamDeckError> {
        Ok(tokio::task::block_in_place(move || ImageRect::from_image(image))?)
    }
}