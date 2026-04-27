#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub width: u16,
    pub height: u16,
    pub layers: Vec<Layer>,
    pub frames: Vec<Frame>,
    pub cels: Vec<Cel>,
}

#[cfg(feature = "image")]
impl Document {
    pub fn render(&self) -> image::RgbaImage {
        let mut base = image::RgbaImage::new(self.width as u32, self.height as u32);
        if self.frames.is_empty() {
            return base;
        }

        // Collect all cels for frame 0
        let mut frame_cels: Vec<&Cel> = self.cels.iter()
            .filter(|c| c.frame_index == 0)
            .collect();

        // Sort cels by layer index (assuming lower index is bottom layer or vice versa, typically we render bottom-up)
        // Let's assume layer index 0 is bottom, and higher is on top. If needed, reverse.
        // Usually, layers are stored bottom-to-top or top-to-bottom. We'll sort by layer_index, assuming 0 is bottom.
        // Actually, typical formats might vary. Aseprite usually stores layers bottom-to-top.
        // We will render in order of layer index ascending, or according to how layers are defined.
        // Let's sort by layer index ascending.
        frame_cels.sort_by_key(|c| c.layer_index);

        for cel in frame_cels {
            let layer = match self.layers.get(cel.layer_index) {
                Some(l) => l,
                None => continue,
            };
            if !layer.visible {
                continue;
            }

            let cel_img: image::RgbaImage = cel.image.clone().into();

            // Note: simple alpha blending via overlay. Opacity/blend modes are complex to implement fully,
            // but we can apply basic overlay which handles alpha. Layer opacity is ignored in this simple
            // implementation, unless we manually modulate the alpha channel of cel_img first.
            let mut cel_img_modulated = cel_img;
            if layer.opacity < 255 {
                for pixel in cel_img_modulated.pixels_mut() {
                    pixel[3] = ((pixel[3] as u32 * layer.opacity as u32) / 255) as u8;
                }
            }

            image::imageops::overlay(&mut base, &cel_img_modulated, cel.x as i64, cel.y as i64);
        }

        base
    }
}

#[cfg(feature = "tiny-skia")]
impl Document {
    pub fn render_skia(&self) -> tiny_skia::Pixmap {
        let mut base = tiny_skia::Pixmap::new(self.width as u32, self.height as u32)
            .expect("Failed to create base Pixmap");

        if self.frames.is_empty() {
            return base;
        }

        let mut frame_cels: Vec<&Cel> = self.cels.iter()
            .filter(|c| c.frame_index == 0)
            .collect();

        frame_cels.sort_by_key(|c| c.layer_index);

        for cel in frame_cels {
            let layer = match self.layers.get(cel.layer_index) {
                Some(l) => l,
                None => continue,
            };
            if !layer.visible {
                continue;
            }

            let cel_img: tiny_skia::Pixmap = cel.image.clone().into();

            let mut paint = tiny_skia::PixmapPaint::default();
            paint.opacity = layer.opacity as f32 / 255.0;

            let transform = tiny_skia::Transform::from_translate(cel.x as f32, cel.y as f32);

            base.draw_pixmap(
                0,
                0,
                cel_img.as_ref(),
                &paint,
                transform,
                None
            );
        }

        base
    }
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

#[cfg(feature = "tiny-skia")]
impl From<Image> for tiny_skia::Pixmap {
    fn from(img: Image) -> Self {
        let mut pixmap = tiny_skia::Pixmap::new(img.width as u32, img.height as u32)
            .expect("Failed to create Pixmap");

        let pixels = pixmap.pixels_mut();
        for (i, chunk) in img.rgba.chunks_exact(4).enumerate() {
            let r = chunk[0];
            let g = chunk[1];
            let b = chunk[2];
            let a = chunk[3];
            // tiny-skia uses PremultipliedColorU8. We need to actively premultiply the colors.
            let color = tiny_skia::ColorU8::from_rgba(r, g, b, a).premultiply();
            pixels[i] = color;
        }
        pixmap
    }
}
