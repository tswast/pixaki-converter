use anyhow::Result;
use aseprite::{AsepriteFile, CelKind};
use pixel_art::{BlendMode, Cel, Document, Frame, Image, Layer};

pub fn parse(aseprite_file: AsepriteFile) -> Result<Document> {
    let mut layers = Vec::new();
    for ase_layer in aseprite_file.layers() {
        let blend_mode = match ase_layer.blend_mode {
            aseprite::BlendMode::Normal => BlendMode::Normal,
            aseprite::BlendMode::Multiply => BlendMode::Multiply,
            aseprite::BlendMode::Screen => BlendMode::Screen,
            aseprite::BlendMode::Overlay => BlendMode::Overlay,
            aseprite::BlendMode::Darken => BlendMode::Darken,
            aseprite::BlendMode::Lighten => BlendMode::Lighten,
            aseprite::BlendMode::ColorDodge => BlendMode::ColorDodge,
            aseprite::BlendMode::ColorBurn => BlendMode::ColorBurn,
            aseprite::BlendMode::HardLight => BlendMode::HardLight,
            aseprite::BlendMode::SoftLight => BlendMode::SoftLight,
            aseprite::BlendMode::Difference => BlendMode::Difference,
            aseprite::BlendMode::Exclusion => BlendMode::Exclusion,
            aseprite::BlendMode::Hue => BlendMode::Hue,
            aseprite::BlendMode::Saturation => BlendMode::Saturation,
            aseprite::BlendMode::Color => BlendMode::Color,
            aseprite::BlendMode::Luminosity => BlendMode::Luminosity,
            _ => BlendMode::Normal, // Fallback for unsupported modes
        };

        layers.push(Layer {
            name: ase_layer.name.clone(),
            opacity: ase_layer.opacity,
            visible: ase_layer.visible,
            blend_mode,
        });
    }

    let mut frames = Vec::new();
    for ase_frame in aseprite_file.frames() {
        frames.push(Frame {
            duration_ms: ase_frame.duration_ms as u32,
        });
    }

    let mut cels = Vec::new();
    let mut images = Vec::new();

    // We iterate over frames then layers so they're in a deterministic order.
    // By using `resolve_cel`, we correctly follow Linked Cels in Aseprite to their source frame.
    for frame_idx in 0..aseprite_file.frames().len() {
        for layer_idx in 0..aseprite_file.layers().len() {
            if let Some(ase_layer_ref) = aseprite_file.layer_ref(layer_idx) {
                if let Some(ase_cel) = aseprite_file.resolve_cel(ase_layer_ref, frame_idx) {
                    let (pixels, x, y) = match &ase_cel.kind {
                        CelKind::Raw { pixels, x, y } => (pixels, *x, *y),
                        CelKind::Compressed { pixels, x, y, .. } => (pixels, *x, *y),
                        _ => continue, // Ignore tilemap cels for now if they don't have pixel data directly
                    };

                    let image_index = if let Some(idx) = images.iter().position(|img: &Image| {
                        img.width == pixels.width && img.height == pixels.height && img.rgba == pixels.data
                    }) {
                        idx
                    } else {
                        let idx = images.len();
                        images.push(Image {
                            width: pixels.width,
                            height: pixels.height,
                            rgba: pixels.data.clone(),
                        });
                        idx
                    };

                    cels.push(Cel {
                        frame_index: frame_idx,
                        layer_index: layer_idx,
                        x,
                        y,
                        image_index,
                    });
                }
            }
        }
    }

    Ok(Document {
        width: aseprite_file.width(),
        height: aseprite_file.height(),
        layers,
        frames,
        cels,
        images,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use aseprite::{ColorMode, Pixels};

    #[test]
    fn test_aseprite_reader() {
        let mut aseprite = AsepriteFile::new(16, 16, ColorMode::Rgba);
        let layer_ref = aseprite.add_layer("Layer 1");
        let frame_idx = aseprite.add_frame(100);
        let pixels = Pixels::new(vec![255; 16 * 16 * 4], 16, 16, ColorMode::Rgba).unwrap();
        aseprite.set_cel(layer_ref, frame_idx, pixels, 0, 0).unwrap();

        let doc = parse(aseprite).unwrap();

        assert_eq!(doc.width, 16);
        assert_eq!(doc.height, 16);
        assert_eq!(doc.layers.len(), 1);
        assert_eq!(doc.layers[0].name, "Layer 1");
        assert_eq!(doc.frames.len(), 1);
        assert_eq!(doc.frames[0].duration_ms, 100);
        assert_eq!(doc.cels.len(), 1);
        assert_eq!(doc.cels[0].x, 0);
        assert_eq!(doc.cels[0].y, 0);
        assert_eq!(doc.images.len(), 1);
        assert_eq!(doc.images[doc.cels[0].image_index].width, 16);
        assert_eq!(doc.images[doc.cels[0].image_index].height, 16);
    }
}
