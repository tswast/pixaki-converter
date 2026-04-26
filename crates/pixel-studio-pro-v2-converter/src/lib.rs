use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as b64, Engine};
use image::{Rgba, RgbaImage};
use pixel_art::{BlendMode, Cel, Document, Frame, Image, Layer};
use pixel_studio_pro_v2::{self, History};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct MetaData {
    rect: Option<RectData>,
    pixels: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct RectData {
    from: PointData,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct PointData {
    x: i32,
    y: i32,
}

fn flood_fill(img: &mut RgbaImage, x: u32, y: u32, fill_color: Rgba<u8>) {
    if x >= img.width() || y >= img.height() {
        return;
    }
    let target_color = *img.get_pixel(x, y);
    if target_color == fill_color {
        return;
    }

    let mut stack = vec![(x, y)];
    while let Some((cx, cy)) = stack.pop() {
        if cx < img.width() && cy < img.height() {
            let current_color = *img.get_pixel(cx, cy);
            if current_color == target_color {
                img.put_pixel(cx, cy, fill_color);

                if cx > 0 { stack.push((cx - 1, cy)); }
                if cx + 1 < img.width() { stack.push((cx + 1, cy)); }
                if cy > 0 { stack.push((cx, cy - 1)); }
                if cy + 1 < img.height() { stack.push((cx, cy + 1)); }
            }
        }
    }
}

pub fn convert(doc: pixel_studio_pro_v2::Document) -> Result<Document> {
    let mut layers: Vec<Layer> = Vec::new();
    let mut frames: Vec<Frame> = Vec::new();
    let mut cels: Vec<Cel> = Vec::new();

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

    // Store the last cel index for each layer to allow O(1) linked cel lookup
    let mut last_cel_per_layer: Vec<Option<usize>> = vec![None; layers.len()];

    let doc_width = doc.width as u32;
    let doc_height = doc.height as u32;

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
                // Find the cel for the same layer in a previous frame
                if let Some(last_cel_idx) = last_cel_per_layer[layer_index] {
                    let prev_x = cels[last_cel_idx].x;
                    let prev_y = cels[last_cel_idx].y;
                    let prev_img = cels[last_cel_idx].image.clone();

                    let new_cel = Cel {
                        frame_index,
                        layer_index,
                        x: prev_x,
                        y: prev_y,
                        image: prev_img,
                    };
                    last_cel_per_layer[layer_index] = Some(cels.len());
                    cels.push(new_cel);
                }
            } else if let Some(history_str) = &psp_layer.history_json {
                let history = serde_json::from_str::<History>(history_str)
                    .map_err(|e| anyhow!("Failed to parse history JSON for layer {}: {}", layer_index, e))?;

                // First pass: load source if available, find min/max boundaries
                let mut min_x: i32 = 0;
                let mut min_y: i32 = 0;
                let mut max_x: i32 = doc_width as i32;
                let mut max_y: i32 = doc_height as i32;

                let mut source_img_opt = None;
                if let Some(source_b64) = history.source {
                    if let Ok(img_data) = b64.decode(&source_b64) {
                        if let Ok(img) = image::load_from_memory(&img_data) {
                            let rgba = img.to_rgba8();
                            if (rgba.width() as i32) > max_x { max_x = rgba.width() as i32; }
                            if (rgba.height() as i32) > max_y { max_y = rgba.height() as i32; }
                            source_img_opt = Some(rgba);
                        }
                    }
                }

                let history_index = history.index as usize;
                for action in history.actions.iter().take(history_index) {
                    if action.tool == 20 || action.tool == 21 || action.tool == 6 {
                        if let Some(meta_str) = &action.meta {
                            if let Ok(meta) = serde_json::from_str::<MetaData>(meta_str) {
                                if let (Some(pixels_b64), Some(rect)) = (&meta.pixels, &meta.rect) {
                                    if let Ok(img_data) = b64.decode(pixels_b64) {
                                        if let Ok(img) = image::load_from_memory(&img_data) {
                                            let dst_min_x = rect.from.x;
                                            let dst_min_y = rect.from.y;
                                            let dst_max_x = rect.from.x + img.width() as i32;
                                            let dst_max_y = rect.from.y + img.height() as i32;
                                            if dst_min_x < min_x { min_x = dst_min_x; }
                                            if dst_min_y < min_y { min_y = dst_min_y; }
                                            if dst_max_x > max_x { max_x = dst_max_x; }
                                            if dst_max_y > max_y { max_y = dst_max_y; }
                                        }
                                    }
                                }
                            }
                        }
                    } else if action.tool == 0 || action.tool == 1 || action.tool == 2 || action.tool == 3 {
                        let pos_bytes = b64.decode(&action.positions).unwrap_or_default();
                        for j in (0..pos_bytes.len()).step_by(4) {
                            if j + 3 < pos_bytes.len() {
                                let px = i16::from_le_bytes([pos_bytes[j], pos_bytes[j + 1]]) as i32;
                                let py = i16::from_le_bytes([pos_bytes[j + 2], pos_bytes[j + 3]]) as i32;
                                if px < min_x { min_x = px; }
                                if py < min_y { min_y = py; }
                                if px + 1 > max_x { max_x = px + 1; }
                                if py + 1 > max_y { max_y = py + 1; }
                            }
                        }
                    }
                }

                // Cap max dimensions to a reasonable safeguard (e.g. 4096) to avoid OOM panics
                let img_width = (max_x - min_x).clamp(1, 4096) as u32;
                let img_height = (max_y - min_y).clamp(1, 4096) as u32;

                let mut final_img = RgbaImage::new(img_width, img_height);
                let mut has_data = false;

                // Draw source image first if available
                if let Some(src_img) = source_img_opt {
                    let offset_x = -min_x;
                    let offset_y = -min_y;
                    for y in 0..src_img.height() {
                        for x in 0..src_img.width() {
                            let dst_x = offset_x + x as i32;
                            let dst_y = offset_y + y as i32;
                            if dst_x >= 0 && dst_y >= 0 && (dst_x as u32) < img_width && (dst_y as u32) < img_height {
                                final_img.put_pixel(dst_x as u32, dst_y as u32, *src_img.get_pixel(x, y));
                                has_data = true;
                            }
                        }
                    }
                }

                // Second pass: replay actions onto the sized canvas
                for action in history.actions.iter().take(history_index) {
                    if action.tool == 20 || action.tool == 21 || action.tool == 6 {
                        // Import/Paste/Move
                        if let Some(meta_str) = &action.meta {
                            if let Ok(meta) = serde_json::from_str::<MetaData>(meta_str) {
                                if let (Some(pixels_b64), Some(rect)) = (meta.pixels, meta.rect) {
                                    if let Ok(img_data) = b64.decode(&pixels_b64) {
                                        if let Ok(img) = image::load_from_memory(&img_data) {
                                            let rgba_patch = img.to_rgba8();
                                            let start_x = rect.from.x - min_x;
                                            let start_y = rect.from.y - min_y;

                                            for y in 0..rgba_patch.height() {
                                                for x in 0..rgba_patch.width() {
                                                    let dst_x = start_x + (x as i32);
                                                    let dst_y = start_y + (y as i32);

                                                    if dst_x >= 0 && dst_y >= 0 && (dst_x as u32) < img_width && (dst_y as u32) < img_height {
                                                        let p = rgba_patch.get_pixel(x, y);
                                                        // over blend or just copy if destination is transparent
                                                        if p[3] > 0 {
                                                            final_img.put_pixel(dst_x as u32, dst_y as u32, *p);
                                                            has_data = true;
                                                        } else if action.tool == 6 {
                                                            // Tool 6 (Move) often includes an erase operation in its rect footprint
                                                            final_img.put_pixel(dst_x as u32, dst_y as u32, Rgba([0, 0, 0, 0]));
                                                            has_data = true;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if action.tool == 0 || action.tool == 3 {
                        // Pen or Bucket Fill
                        let pos_bytes = b64.decode(&action.positions).unwrap_or_default();
                        let col_bytes = b64.decode(&action.colors).unwrap_or_default();

                        if col_bytes.len() >= 4 {
                            let color = Rgba([col_bytes[0], col_bytes[1], col_bytes[2], col_bytes[3]]);
                            for j in (0..pos_bytes.len()).step_by(4) {
                                if j + 3 < pos_bytes.len() {
                                    let px = i16::from_le_bytes([pos_bytes[j], pos_bytes[j + 1]]) as i32 - min_x;
                                    let py = i16::from_le_bytes([pos_bytes[j + 2], pos_bytes[j + 3]]) as i32 - min_y;

                                    if px >= 0 && py >= 0 && (px as u32) < img_width && (py as u32) < img_height {
                                        if action.tool == 0 {
                                            final_img.put_pixel(px as u32, py as u32, color);
                                        } else {
                                            flood_fill(&mut final_img, px as u32, py as u32, color);
                                        }
                                        has_data = true;
                                    }
                                }
                            }
                        }
                    } else if action.tool == 1 || action.tool == 2 {
                        // Eraser or Bucket Erase
                        let pos_bytes = b64.decode(&action.positions).unwrap_or_default();
                        for j in (0..pos_bytes.len()).step_by(4) {
                            if j + 3 < pos_bytes.len() {
                                let px = i16::from_le_bytes([pos_bytes[j], pos_bytes[j + 1]]) as i32 - min_x;
                                let py = i16::from_le_bytes([pos_bytes[j + 2], pos_bytes[j + 3]]) as i32 - min_y;

                                if px >= 0 && py >= 0 && (px as u32) < img_width && (py as u32) < img_height {
                                    if action.tool == 1 {
                                        final_img.put_pixel(px as u32, py as u32, Rgba([0, 0, 0, 0]));
                                    } else {
                                        flood_fill(&mut final_img, px as u32, py as u32, Rgba([0, 0, 0, 0]));
                                    }
                                    has_data = true;
                                }
                            }
                        }
                    }
                }

                if has_data {
                    let cel = Cel {
                        frame_index,
                        layer_index,
                        x: (psp_layer.sx + min_x).clamp(i16::MIN as i32, i16::MAX as i32) as i16,
                        y: (psp_layer.sy + min_y).clamp(i16::MIN as i32, i16::MAX as i32) as i16,
                        image: Image {
                            width: u16::try_from(img_width).unwrap_or(u16::MAX),
                            height: u16::try_from(img_height).unwrap_or(u16::MAX),
                            rgba: final_img.into_raw(),
                        },
                    };

                    last_cel_per_layer[layer_index] = Some(cels.len());
                    cels.push(cel);
                }
            }
        }
    }

    Ok(Document {
        width: u16::try_from(doc.width).unwrap_or(u16::MAX),
        height: u16::try_from(doc.height).unwrap_or(u16::MAX),
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
