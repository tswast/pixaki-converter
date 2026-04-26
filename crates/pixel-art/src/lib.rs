#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub width: u16,
    pub height: u16,
    pub layers: Vec<Layer>,
    pub frames: Vec<Frame>,
    pub cels: Vec<Cel>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub name: String,
    pub opacity: u8,
    pub visible: bool,
    pub blend_mode: BlendMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub duration_ms: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cel {
    pub frame_index: usize,
    pub layer_index: usize,
    pub x: i16,
    pub y: i16,
    pub image: Image,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub width: u16,
    pub height: u16,
    pub rgba: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document {
            width: 10,
            height: 10,
            layers: vec![],
            frames: vec![],
            cels: vec![],
        };
        assert_eq!(doc.width, 10);
        assert_eq!(doc.height, 10);
    }
}

#[cfg(feature = "image")]
impl From<Image> for image::RgbaImage {
    fn from(img: Image) -> Self {
        image::RgbaImage::from_vec(img.width as u32, img.height as u32, img.rgba)
            .expect("Buffer size should match dimensions")
    }
}

#[cfg(feature = "image")]
impl From<image::RgbaImage> for Image {
    fn from(img: image::RgbaImage) -> Self {
        Self {
            width: img.width().try_into().expect("image width exceeds u16"),
            height: img.height().try_into().expect("image height exceeds u16"),
            rgba: img.into_raw(),
        }
    }
}
