use std::path::Path;
use pixel_art::{Document, Layer, Frame, Cel, BlendMode, Image};
use pixaki_v2;
use anyhow::Result;

pub fn convert(doc: pixaki_v2::Document, base_path: &Path) -> Result<Document> {
    let width = doc.size.width as u16;
    let height = doc.size.height as u16;
    let symbols = &doc.symbols;

    let mut layers = Vec::new();
    let mut frames = Vec::new();
    let mut cels = Vec::new();

    if let Some(symbol) = symbols.get(0) {
        // Add layers based on the first frame
        if let Some(first_frame) = symbol.frames.get(0) {
            for (i, layer) in first_frame.layers.iter().enumerate() {
                layers.push(Layer {
                    name: format!("Layer {}", i),
                    opacity: (layer.alpha * 255.0) as u8,
                    visible: layer.visible,
                    blend_mode: BlendMode::Normal,
                });
            }
        }

        // Add frames and cels
        for (frame_index, frame_v2) in symbol.frames.iter().enumerate() {
            frames.push(Frame {
                duration_ms: frame_v2.duration,
            });

            for (layer_index, layer_v2) in frame_v2.layers.iter().enumerate() {
                if layer_index < layers.len() {
                    let image_path = base_path.join(&layer_v2.image_filename);
                    if let Ok(img) = image::open(&image_path) {
                        let rgba_img = img.to_rgba8();
                        let (img_width, img_height) = rgba_img.dimensions();

                        cels.push(Cel {
                            frame_index,
                            layer_index,
                            x: 0,
                            y: 0,
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

    Ok(Document {
        width,
        height,
        layers,
        frames,
        cels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pixaki_v2;

    #[test]
    fn test_v2_conversion_basic() {
        let doc_v2 = pixaki_v2::Document {
            size: pixaki_v2::Size { width: 16.0, height: 16.0 },
            symbols: vec![
                pixaki_v2::Symbol {
                    name: "Symbol 1".to_string(),
                    frames: vec![
                        pixaki_v2::Frame {
                            duration: 100,
                            layers: vec![
                                pixaki_v2::Layer {
                                    alpha: 1.0,
                                    image_filename: "non_existent.png".to_string(),
                                    visible: true,
                                    ..Default::default()
                                }
                            ],
                            ..Default::default()
                        }
                    ],
                    ..Default::default()
                }
            ],
            ..Default::default()
        };

        let result = convert(doc_v2, Path::new(".")).unwrap();
        assert_eq!(result.width, 16);
        assert_eq!(result.layers.len(), 1);
        assert_eq!(result.frames.len(), 1);
        // Cels will be 0 because we don't have the image file
        assert_eq!(result.cels.len(), 0);
    }
}
