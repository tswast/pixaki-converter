use anyhow::Result;
use aseprite::{AsepriteFile, BlendMode as AseBlendMode, ColorMode, LayerOptions, Pixels};
use pixel_art::{BlendMode, Document};

pub mod reader;

pub fn convert(doc: Document) -> Result<AsepriteFile> {
    let mut aseprite = AsepriteFile::new(doc.width, doc.height, ColorMode::Rgba);

    let mut layer_handles = Vec::new();
    for layer in &doc.layers {
        let opts = LayerOptions {
            opacity: layer.opacity,
            blend_mode: map_blend_mode(layer.blend_mode),
            visible: layer.visible,
            ..Default::default()
        };
        let handle = aseprite.add_layer_with(&layer.name, opts);
        layer_handles.push(handle);
    }

    let mut frame_handles = Vec::new();
    for frame in &doc.frames {
        let handle = aseprite.add_frame(frame.duration_ms as u16);
        frame_handles.push(handle);
    }

    for cel in &doc.cels {
        let layer_handle = layer_handles[cel.layer_index];
        let frame_handle = frame_handles[cel.frame_index];
        let image = &doc.images[cel.image_index];
        let pixels = Pixels::new(
            image.rgba.clone(),
            image.width,
            image.height,
            ColorMode::Rgba,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create Pixels: {}", e))?;
        aseprite
            .set_cel(layer_handle, frame_handle, pixels, cel.x, cel.y)
            .map_err(|e| anyhow::anyhow!("Failed to set cel: {}", e))?;
    }

    Ok(aseprite)
}

fn map_blend_mode(b: BlendMode) -> AseBlendMode {
    match b {
        BlendMode::Normal => AseBlendMode::Normal,
        BlendMode::Multiply => AseBlendMode::Multiply,
        BlendMode::Screen => AseBlendMode::Screen,
        BlendMode::Overlay => AseBlendMode::Overlay,
        BlendMode::Darken => AseBlendMode::Darken,
        BlendMode::Lighten => AseBlendMode::Lighten,
        BlendMode::ColorDodge => AseBlendMode::ColorDodge,
        BlendMode::ColorBurn => AseBlendMode::ColorBurn,
        BlendMode::HardLight => AseBlendMode::HardLight,
        BlendMode::SoftLight => AseBlendMode::SoftLight,
        BlendMode::Difference => AseBlendMode::Difference,
        BlendMode::Exclusion => AseBlendMode::Exclusion,
        BlendMode::Hue => AseBlendMode::Hue,
        BlendMode::Saturation => AseBlendMode::Saturation,
        BlendMode::Color => AseBlendMode::Color,
        BlendMode::Luminosity => AseBlendMode::Luminosity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pixel_art::{BlendMode, Document, Frame, Layer};

    #[test]
    fn test_aseprite_conversion_basic() {
        let doc = Document {
            width: 8,
            height: 8,
            layers: vec![Layer {
                name: "Base".to_string(),
                opacity: 255,
                visible: true,
                blend_mode: BlendMode::Normal,
            }],
            frames: vec![Frame { duration_ms: 100 }],
            cels: vec![],
            images: vec![],
        };

        let ase = convert(doc).unwrap();
        assert_eq!(ase.width(), 8);
        assert_eq!(ase.height(), 8);
        assert_eq!(ase.frames().len(), 1);
        assert_eq!(ase.layers().len(), 1);
    }
}
