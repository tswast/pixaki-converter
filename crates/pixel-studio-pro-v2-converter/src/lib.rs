use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as b64, Engine};
use pixel_art::{BlendMode, Cel, Document, Frame, Image, Layer};
use pixel_studio_pro_v2::{self, History};

pub fn convert(doc: pixel_studio_pro_v2::Document) -> Result<Document> {
    let mut layers = Vec::new();
    let mut frames = Vec::new();
    let mut cels = Vec::new();

    let clip = doc
        .clips
        .first()
        .ok_or_else(|| anyhow!("No clips found in document"))?;

    // Create global layers from the first frame
    if let Some(first_frame) = clip.frames.first() {
        for psp_layer in &first_frame.layers {
            layers.push(Layer {
                name: psp_layer.name.clone(),
                opacity: (psp_layer.opacity * 255.0).clamp(0.0, 255.0) as u8,
                visible: !psp_layer.hidden,
                blend_mode: BlendMode::Normal, // Basic fallback
            });
        }
    }

    // Process frames and cels
    for (frame_index, psp_frame) in clip.frames.iter().enumerate() {
        frames.push(Frame {
            duration_ms: (psp_frame.delay * 1000.0).round() as u32,
        });

        for (layer_index, psp_layer) in psp_frame.layers.iter().enumerate() {
            if layer_index >= layers.len() {
                continue;
            }

            if psp_layer.linked {
                // Find the cel for the same layer in the previous frame
                if frame_index > 0 {
                    let new_cel = cels
                        .iter()
                        .find(|c: &&Cel| c.frame_index == frame_index - 1 && c.layer_index == layer_index)
                        .map(|prev_cel| Cel {
                            frame_index,
                            layer_index,
                            x: prev_cel.x,
                            y: prev_cel.y,
                            image: prev_cel.image.clone(),
                        });

                    if let Some(cel) = new_cel {
                        cels.push(cel);
                    }
                }
            } else if let Some(history_str) = &psp_layer.history_json {
                let history = serde_json::from_str::<History>(history_str)
                    .map_err(|e| anyhow!("Failed to parse history JSON for layer {}: {}", layer_index, e))?;

                if let Some(source_b64) = history.source {
                    let img_data = b64.decode(&source_b64)
                        .map_err(|e| anyhow!("Failed to decode base64 source for layer {}: {}", layer_index, e))?;

                    let img = image::load_from_memory(&img_data)
                        .map_err(|e| anyhow!("Failed to load image from memory for layer {}: {}", layer_index, e))?;

                    let rgba_img = img.to_rgba8();
                    let (img_width, img_height) = rgba_img.dimensions();
                    cels.push(Cel {
                        frame_index,
                        layer_index,
                        x: psp_layer.sx as i16,
                        y: psp_layer.sy as i16,
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

    Ok(Document {
        width: doc.width,
        height: doc.height,
        layers,
        frames,
        cels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_psp_v2_conversion_basic() {
        let doc_psp = pixel_studio_pro_v2::Document {
            version: 2,
            id: "doc1".to_string(),
            name: "Test Doc".to_string(),
            width: 16,
            height: 16,
            clips: vec![pixel_studio_pro_v2::Clip {
                id: "clip1".to_string(),
                name: "Clip 1".to_string(),
                frames: vec![pixel_studio_pro_v2::Frame {
                    id: "frame1".to_string(),
                    delay: 0.1,
                    layers: vec![pixel_studio_pro_v2::Layer {
                        id: "layer1".to_string(),
                        name: "Layer 1".to_string(),
                        opacity: 1.0,
                        transparency: -1.0,
                        version: 1,
                        ..Default::default()
                    }],
                    active_layer_index: Some(0),
                    ..Default::default()
                }],
                layer_types: vec![0],
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = convert(doc_psp).unwrap();
        assert_eq!(result.width, 16);
        assert_eq!(result.layers.len(), 1);
        assert_eq!(result.frames.len(), 1);
        assert_eq!(result.frames[0].duration_ms, 100);
        assert_eq!(result.cels.len(), 0);
    }
}
