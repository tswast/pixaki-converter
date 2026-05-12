use anyhow::{Result, anyhow};
use pixel_art::{BlendMode, Cel, Document, Frame, Image, Layer};
use psd::Psd;

pub fn convert(psd_bytes: &[u8]) -> Result<Document> {
    let psd = Psd::from_bytes(psd_bytes).map_err(|e| anyhow!("Failed to parse PSD: {:?}", e))?;

    let width = psd
        .width()
        .try_into()
        .map_err(|_| anyhow!("PSD width exceeds u16 max"))?;
    let height = psd
        .height()
        .try_into()
        .map_err(|_| anyhow!("PSD height exceeds u16 max"))?;

    let mut layers = Vec::new();
    let mut cels = Vec::new();
    let mut images = Vec::new();

    for (layer_index, psd_layer) in psd.layers().iter().enumerate() {
        let name = psd_layer.name().to_string();
        let opacity = psd_layer.opacity();
        let visible = psd_layer.visible();

        // `psd::BlendMode` is not publicly exported in psd 0.3.5, but its Debug implementation returns
        // the variant name. We map based on this string representation.
        let blend_mode_str = format!("{:?}", psd_layer.blend_mode());
        let blend_mode = match blend_mode_str.as_str() {
            "Normal" => BlendMode::Normal,
            "Multiply" => BlendMode::Multiply,
            "Screen" => BlendMode::Screen,
            "Overlay" => BlendMode::Overlay,
            "Darken" => BlendMode::Darken,
            "Lighten" => BlendMode::Lighten,
            "ColorDodge" => BlendMode::ColorDodge,
            "ColorBurn" => BlendMode::ColorBurn,
            "HardLight" => BlendMode::HardLight,
            "SoftLight" => BlendMode::SoftLight,
            "Difference" => BlendMode::Difference,
            "Exclusion" => BlendMode::Exclusion,
            "Hue" => BlendMode::Hue,
            "Saturation" => BlendMode::Saturation,
            "Color" => BlendMode::Color,
            "Luminosity" => BlendMode::Luminosity,
            _ => BlendMode::Normal, // Default to normal for unsupported modes
        };

        layers.push(Layer {
            name,
            opacity,
            visible,
            blend_mode,
        });

        let layer_width = psd_layer.width();
        let layer_height = psd_layer.height();

        if layer_width > 0 && layer_height > 0 {
            let rgba = psd_layer.rgba();

            let x = psd_layer.layer_left().try_into().unwrap_or(0);
            let y = psd_layer.layer_top().try_into().unwrap_or(0);

            let image_index = images.len();
            images.push(Image {
                width: layer_width
                    .try_into()
                    .map_err(|_| anyhow!("Layer width exceeds u16 max"))?,
                height: layer_height
                    .try_into()
                    .map_err(|_| anyhow!("Layer height exceeds u16 max"))?,
                rgba,
            });

            cels.push(Cel {
                frame_index: 0,
                layer_index,
                x,
                y,
                image_index,
            });
        }
    }

    // Add a single frame since PSD is static
    let frames = vec![Frame { duration_ms: 100 }];

    Ok(Document {
        width,
        height,
        layers,
        frames,
        cels,
        images,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_psd_data() {
        let result = convert(b"not a valid psd file");
        assert!(result.is_err());
    }
}
