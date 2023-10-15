/// HIDAPI Vendor ID that Elgato products use
pub const ELGATO_VENDOR_ID: u16 = 0x0fd9;

/// Product ID of first revision of original Stream Deck
pub const PID_STREAMDECK_ORIGINAL: u16 = 0x0060;
/// Product ID of second revision of original Stream Deck
pub const PID_STREAMDECK_ORIGINAL_V2: u16 = 0x006d;
/// Product ID of Stream Deck Mini
pub const PID_STREAMDECK_MINI: u16 = 0x0063;
/// Product ID of first revision of Stream Deck XL
pub const PID_STREAMDECK_XL: u16 = 0x006c;
/// Product ID of second revision of Stream Deck XL
pub const PID_STREAMDECK_XL_V2: u16 = 0x008f;
/// Product ID of Stream Deck Mk2
pub const PID_STREAMDECK_MK2: u16 = 0x0080;
/// Product ID of Stream Deck Mini Mk2
pub const PID_STREAMDECK_MINI_MK2: u16 = 0x0090;
/// Product ID of Stream Deck Pedal
pub const PID_STREAMDECK_PEDAL: u16 = 0x0086;
/// Product ID of Stream Deck Plus
pub const PID_STREAMDECK_PLUS: u16 = 0x0084;

/// Enum describing kinds of Stream Decks out there
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Kind {
    /// First revision of original Stream Deck
    Original,
    /// Second revision of original Stream Deck
    OriginalV2,
    /// Stream Deck Mini
    Mini,
    /// First revision of Stream Deck XL
    Xl,
    /// Second revision of Stream Deck XL
    XlV2,
    /// Stream Deck Mk2
    Mk2,
    /// Stream Deck Mini Mk2
    MiniMk2,
    /// Stream Deck Pedal
    Pedal,
    /// Stream Deck Plus
    Plus
}

impl Kind {
    /// Creates [Kind] variant from Product ID
    pub fn from_pid(pid: u16) -> Option<Kind> {
        match pid {
            PID_STREAMDECK_ORIGINAL => Some(Kind::Original),
            PID_STREAMDECK_ORIGINAL_V2 => Some(Kind::OriginalV2),
            PID_STREAMDECK_MINI => Some(Kind::Mini),
            PID_STREAMDECK_XL => Some(Kind::Xl),
            PID_STREAMDECK_XL_V2 => Some(Kind::XlV2),
            PID_STREAMDECK_MK2 => Some(Kind::Mk2),
            PID_STREAMDECK_MINI_MK2 => Some(Kind::MiniMk2),
            PID_STREAMDECK_PEDAL => Some(Kind::Pedal),
            PID_STREAMDECK_PLUS => Some(Kind::Plus),
            _ => None
        }
    }

    /// Retrieves Product ID of the Stream Deck
    pub fn product_id(&self) -> u16 {
        match self {
            Kind::Original => PID_STREAMDECK_ORIGINAL,
            Kind::OriginalV2 => PID_STREAMDECK_ORIGINAL_V2,
            Kind::Mini => PID_STREAMDECK_MINI,
            Kind::Xl => PID_STREAMDECK_XL,
            Kind::XlV2 => PID_STREAMDECK_XL_V2,
            Kind::Mk2 => PID_STREAMDECK_MK2,
            Kind::MiniMk2 => PID_STREAMDECK_MINI_MK2,
            Kind::Pedal => PID_STREAMDECK_PEDAL,
            Kind::Plus => PID_STREAMDECK_PLUS
        }
    }

    /// Retrieves Vendor ID used by Elgato hardware
    pub fn vendor_id(&self) -> u16 {
        ELGATO_VENDOR_ID
    }

    /// Amount of keys the Stream Deck kind has
    pub fn key_count(&self) -> u8 {
        match self {
            Kind::Original | Kind::OriginalV2 | Kind::Mk2 => 15,
            Kind::Mini | Kind::MiniMk2 => 6,
            Kind::Xl | Kind::XlV2 => 32,
            Kind::Pedal => 3,
            Kind::Plus => 8
        }
    }

    /// Amount of button rows the Stream Deck kind has
    pub fn row_count(&self) -> u8 {
        match self {
            Kind::Original | Kind::OriginalV2 | Kind::Mk2 => 3,
            Kind::Mini | Kind::MiniMk2 => 2,
            Kind::Xl | Kind::XlV2 => 4,
            Kind::Pedal => 1,
            Kind::Plus => 2
        }
    }

    /// Amount of button columns the Stream Deck kind has
    pub fn column_count(&self) -> u8 {
        match self {
            Kind::Original | Kind::OriginalV2 | Kind::Mk2 => 5,
            Kind::Mini | Kind::MiniMk2 => 3,
            Kind::Xl | Kind::XlV2 => 8,
            Kind::Pedal => 3,
            Kind::Plus => 4
        }
    }

    /// Amount of encoders/knobs the Stream Deck kind has
    pub fn encoder_count(&self) -> u8 {
        match self {
            Kind::Plus => 4,
            _ => 0,
        }
    }

    /// Size of the LCD strip on the device
    pub fn lcd_strip_size(&self) -> Option<(usize, usize)> {
        match self {
            Kind::Plus => Some((800, 100)),
            _ => None,
        }
    }

    /// Tells if the Stream Deck kind has a screen
    pub fn is_visual(&self) -> bool {
        match self {
            Kind::Pedal => false,
            _ => true,
        }
    }

    /// Key layout of the Stream Deck kind as (rows, columns)
    pub fn key_layout(&self) -> (u8, u8) {
        (self.row_count(), self.column_count())
    }

    /// Image format used by the Stream Deck kind
    pub fn key_image_format(&self) -> ImageFormat {
        match self {
            Kind::Original => ImageFormat {
                mode: ImageMode::BMP,
                size: (72, 72),
                rotation: ImageRotation::Rot0,
                mirror: ImageMirroring::Both
            },

            Kind::OriginalV2 | Kind::Mk2 => ImageFormat {
                mode: ImageMode::JPEG,
                size: (72, 72),
                rotation: ImageRotation::Rot0,
                mirror: ImageMirroring::Both
            },

            Kind::Mini | Kind::MiniMk2 => ImageFormat {
                mode: ImageMode::BMP,
                size: (80, 80),
                rotation: ImageRotation::Rot90,
                mirror: ImageMirroring::Y
            },

            Kind::Xl | Kind::XlV2 => ImageFormat {
                mode: ImageMode::JPEG,
                size: (96, 96),
                rotation: ImageRotation::Rot0,
                mirror: ImageMirroring::Both
            },

            Kind::Plus => ImageFormat {
                mode: ImageMode::JPEG,
                size: (120, 120),
                rotation: ImageRotation::Rot0,
                mirror: ImageMirroring::None
            },

            Kind::Pedal => ImageFormat::default(),
        }
    }
}

/// Image format used by the Stream Deck
#[derive(Copy, Clone, Debug, Hash)]
pub struct ImageFormat {
    /// Image format/mode
    pub mode: ImageMode,
    /// Image size
    pub size: (usize, usize),
    /// Image rotation
    pub rotation: ImageRotation,
    /// Image mirroring
    pub mirror: ImageMirroring,
}

impl Default for ImageFormat {
    fn default() -> Self {
        Self {
            mode: ImageMode::None,
            size: (0, 0),
            rotation: ImageRotation::Rot0,
            mirror: ImageMirroring::None
        }
    }
}

/// Image rotation
#[derive(Copy, Clone, Debug, Hash)]
pub enum ImageRotation {
    /// No rotation
    Rot0,
    /// 90 degrees clockwise
    Rot90,
    /// 180 degrees
    Rot180,
    /// 90 degrees counter-clockwise
    Rot270,
}

/// Image mirroring
#[derive(Copy, Clone, Debug, Hash)]
pub enum ImageMirroring {
    /// No image mirroring
    None,
    /// Flip by X
    X,
    /// Flip by Y
    Y,
    /// Flip by both axes
    Both
}

/// Image format
#[derive(Copy, Clone, Debug, Hash)]
pub enum ImageMode {
    /// No image
    None,
    /// Bitmap image
    BMP,
    /// Jpeg image
    JPEG
}