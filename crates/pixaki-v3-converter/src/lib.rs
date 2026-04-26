use std::collections::HashMap;
use std::path::Path;
use pixel_art::{Document, Layer, Frame, Cel, BlendMode, Image};
use pixaki_v3;
use anyhow::{Result, Context};

pub fn convert(doc: pixaki_v3::Document, pixaki_path: &Path) -> Result<Document> {
    let sprite = doc.sprites.get(0).context("No sprite found")?;
    let width = sprite.size.width as u16;
    let height = sprite.size.height as u16;
    let num_frames = sprite.duration;

    let mut layers = Vec::new();
    for layer in &sprite.layers {
        layers.push(Layer {
            name: layer.name.clone(),
            opacity: (layer.opacity * 255.0) as u8,
            visible: layer.is_visible,
            blend_mode: map_blend_mode(layer.blend_mode.as_deref().unwrap_or("normal")),
        });
    }

    let mut frames = Vec::new();
    for _ in 0..num_frames {
        frames.push(Frame {
            duration_ms: 100, // Default duration if not specified per frame in pixaki v3?
        });
    }

    let cel_map: HashMap<_, _> = sprite
        .cels
        .iter()
        .map(|c| (c.identifier.clone(), c))
        .collect();

    let image_dir = if pixaki_path.join("images").join("drawings").is_dir() {
        pixaki_path.join("images").join("drawings")
    } else {
        pixaki_path.to_path_buf()
    };

    let mut cels = Vec::new();
    for frame_index in 0..num_frames {
        for (layer_index, layer) in sprite.layers.iter().enumerate() {
            for clip in &layer.clips {
                let in_range = match &clip.range {
                    Some(range) => frame_index >= range.start && frame_index < range.end,
                    None => frame_index == 0,
                };

                if in_range {
                    if let Some(cel_info) = cel_map.get(&clip.item_identifier) {
                        let image_path = image_dir.join(format!("{}.png", cel_info.identifier));
                        if let Ok(img) = image::open(&image_path) {
                            let rgba_img = img.to_rgba8();
                            let (img_width, img_height) = rgba_img.dimensions();
                            let x = cel_info.frame[0][0] as i16;
                            let y = cel_info.frame[0][1] as i16;

                            cels.push(Cel {
                                frame_index: frame_index as usize,
                                layer_index,
                                x,
                                y,
                                image: Image {
                                    width: img_width as u16,
                                    height: img_height as u16,
                                    rgba: rgba_img.into_raw(),
                                },
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(Document {
        width,
        height,
        layers,
        frames,
        cels,
    })
}

fn map_blend_mode(s: &str) -> BlendMode {
    match s {
        "normal" => BlendMode::Normal,
        "multiply" => BlendMode::Multiply,
        "screen" => BlendMode::Screen,
        "overlay" => BlendMode::Overlay,
        "darken" => BlendMode::Darken,
        "lighten" => BlendMode::Lighten,
        "colorDodge" => BlendMode::ColorDodge,
        "colorBurn" => BlendMode::ColorBurn,
        "hardLight" => BlendMode::HardLight,
        "softLight" => BlendMode::SoftLight,
        "difference" => BlendMode::Difference,
        "exclusion" => BlendMode::Exclusion,
        "hue" => BlendMode::Hue,
        "saturation" => BlendMode::Saturation,
        "color" => BlendMode::Color,
        "luminosity" => BlendMode::Luminosity,
        _ => BlendMode::Normal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pixaki_v3;

    #[test]
    fn test_v3_conversion_basic() {
        let doc_v3 = pixaki_v3::Document {
            sprites: vec![
                pixaki_v3::Sprite {
                    size: pixaki_v3::Size { width: 32.0, height: 32.0 },
                    duration: 2,
                    layers: vec![
                        pixaki_v3::Layer {
                            name: "Layer 1".to_string(),
                            is_visible: true,
                            opacity: 1.0,
                            blend_mode: Some("normal".to_string()),
                            ..Default::default()
                        }
                    ],
                    ..Default::default()
                }
            ],
            ..Default::default()
        };

        let result = convert(doc_v3, Path::new(".")).unwrap();
        assert_eq!(result.width, 32);
        assert_eq!(result.layers.len(), 1);
        assert_eq!(result.frames.len(), 2);
    }
}
