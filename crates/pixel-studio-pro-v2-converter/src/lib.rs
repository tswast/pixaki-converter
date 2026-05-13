use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose::STANDARD as b64};
use image::{Rgba, RgbaImage};
use pixel_art::{BlendMode, Cel, Document, Frame, Image, Layer};
use pixel_studio_pro_v2::{self, History};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct MetaData {
    rect: Option<RectData>,
    pixels: Option<String>,
    from: Option<PointData>,
    to: Option<PointData>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct RectData {
    from: PointData,
    to: Option<PointData>,
    width: Option<i32>,
    height: Option<i32>,
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

                if cx > 0 {
                    stack.push((cx - 1, cy));
                }
                if cx + 1 < img.width() {
                    stack.push((cx + 1, cy));
                }
                if cy > 0 {
                    stack.push((cx, cy - 1));
                }
                if cy + 1 < img.height() {
                    stack.push((cx, cy + 1));
                }
            }
        }
    }
}

fn update_bounds_from_positions(
    positions_b64: &str,
    doc_height: u32,
    min_x: &mut i32,
    min_y: &mut i32,
    max_x: &mut i32,
    max_y: &mut i32,
) {
    use base64::{Engine as _, engine::general_purpose};
    let pos_bytes = general_purpose::STANDARD
        .decode(positions_b64)
        .unwrap_or_default();
    for j in (0..pos_bytes.len()).step_by(4) {
        if j + 3 < pos_bytes.len() {
            let px = i16::from_le_bytes([pos_bytes[j], pos_bytes[j + 1]]) as i32;
            let py = doc_height as i32
                - 1
                - i16::from_le_bytes([pos_bytes[j + 2], pos_bytes[j + 3]]) as i32;
            if px < *min_x {
                *min_x = px;
            }
            if py < *min_y {
                *min_y = py;
            }
            if px > *max_x {
                *max_x = px;
            }
            if py > *max_y {
                *max_y = py;
            }
        }
    }
}

fn calculate_bounds(
    history: &History,
    doc_width: u32,
    doc_height: u32,
) -> (i32, i32, i32, i32, Option<RgbaImage>) {
    let mut min_x: i32 = 0;
    let mut min_y: i32 = 0;
    let mut max_x: i32 = doc_width as i32;
    let mut max_y: i32 = doc_height as i32;

    let mut source_img_opt = None;
    if let Some(source_b64) = &history.source {
        if let Ok(img_data) = b64.decode(source_b64) {
            if let Ok(img) = image::load_from_memory(&img_data) {
                let rgba = img.to_rgba8();
                if (rgba.width() as i32) > max_x {
                    max_x = rgba.width() as i32;
                }
                if (rgba.height() as i32) > max_y {
                    max_y = rgba.height() as i32;
                }
                source_img_opt = Some(rgba);
            }
        }
    }

    let history_index = history.index as usize;
    for action in history.actions.iter().take(history_index) {
        if let Ok(tool_type) = pixel_studio_pro_v2::Tool::try_from(action.tool) {
            match tool_type {
                pixel_studio_pro_v2::Tool::RotateLeft | pixel_studio_pro_v2::Tool::RotateRight => {
                    if let Some(meta_str) = &action.meta {
                        if let Ok(meta) = serde_json::from_str::<MetaData>(meta_str) {
                            if let Some(rect) = &meta.rect {
                                let w = rect.width.unwrap_or_else(|| {
                                    rect.to.as_ref().map_or(0, |to| to.x - rect.from.x)
                                });
                                let h = rect.height.unwrap_or_else(|| {
                                    rect.to.as_ref().map_or(0, |to| to.y - rect.from.y)
                                });

                                let dst_min_x = rect.from.x;
                                let dst_max_x = rect.from.x + h; // swapped width/height
                                let dst_max_y = doc_height as i32 - 1 - rect.from.y;
                                let dst_min_y = dst_max_y - w; // swapped width/height

                                if dst_min_x < min_x {
                                    min_x = dst_min_x;
                                }
                                if dst_min_y < min_y {
                                    min_y = dst_min_y;
                                }
                                if dst_max_x > max_x {
                                    max_x = dst_max_x;
                                }
                                if dst_max_y > max_y {
                                    max_y = dst_max_y;
                                }
                            }
                        }
                    }
                }
                pixel_studio_pro_v2::Tool::Move => {
                    let pos_bytes = b64.decode(&action.positions).unwrap_or_default();
                    if pos_bytes.len() >= 8 {
                        let px1 = i16::from_le_bytes([pos_bytes[0], pos_bytes[1]]) as i32;
                        let py1 = doc_height as i32
                            - 1
                            - i16::from_le_bytes([pos_bytes[2], pos_bytes[3]]) as i32;
                        let px2 = i16::from_le_bytes([pos_bytes[4], pos_bytes[5]]) as i32;
                        let py2 = doc_height as i32
                            - 1
                            - i16::from_le_bytes([pos_bytes[6], pos_bytes[7]]) as i32;
                        let dx = px2 - px1;
                        let dy = py2 - py1;

                        if let Some(meta_str) = &action.meta {
                            if let Ok(meta) = serde_json::from_str::<MetaData>(meta_str) {
                                if let (Some(from), Some(to)) = (&meta.from, &meta.to) {
                                    let sel_min_x = from.x.min(to.x);
                                    let sel_max_x = from.x.max(to.x);
                                    let sel_min_y = from.y.min(to.y);
                                    let sel_max_y = from.y.max(to.y);

                                    let top_down_min_y = doc_height as i32 - 1 - sel_max_y;
                                    let top_down_max_y = doc_height as i32 - 1 - sel_min_y;

                                    let shifted_min_x = sel_min_x + dx;
                                    let shifted_max_x = sel_max_x + dx;
                                    let shifted_min_y = top_down_min_y + dy;
                                    let shifted_max_y = top_down_max_y + dy;

                                    if shifted_min_x < min_x {
                                        min_x = shifted_min_x;
                                    }
                                    if shifted_max_x > max_x {
                                        max_x = shifted_max_x;
                                    }
                                    if shifted_min_y < min_y {
                                        min_y = shifted_min_y;
                                    }
                                    if shifted_max_y > max_y {
                                        max_y = shifted_max_y;
                                    }

                                    if sel_min_x < min_x { 
                                        min_x = sel_min_x; 
                                    }
                                    if sel_max_x + 1 > max_x { 
                                        max_x = sel_max_x + 1; 
                                    }
                                    if top_down_min_y < min_y { 
                                        min_y = top_down_min_y; 
                                    }
                                    if top_down_max_y + 1 > max_y { 
                                        max_y = top_down_max_y + 1; 
                                    }
                                }
                            }
                        } else {
                            for j in (8..pos_bytes.len()).step_by(4) {
                                if j + 3 < pos_bytes.len() {
                                    let px = i16::from_le_bytes([pos_bytes[j], pos_bytes[j + 1]]) as i32;
                                    let py = doc_height as i32 - 1 - i16::from_le_bytes([pos_bytes[j + 2], pos_bytes[j + 3]]) as i32;
                                    let shifted_x = px + dx;
                                    let shifted_y = py + dy;

                                    if px < min_x { 
                                        min_x = px; 
                                    }
                                    if px + 1 > max_x { 
                                        max_x = px + 1; 
                                    }
                                    if py < min_y { 
                                        min_y = py; 
                                    }
                                    if py + 1 > max_y { 
                                        max_y = py + 1; 
                                    }

                                    if shifted_x < min_x { 
                                        min_x = shifted_x; 
                                    }
                                    if shifted_x + 1 > max_x { 
                                        max_x = shifted_x + 1; 
                                    }
                                    if shifted_y < min_y { 
                                        min_y = shifted_y; 
                                    }
                                    if shifted_y + 1 > max_y { 
                                        max_y = shifted_y + 1; 
                                    }
                                }
                            }
                        }
                    }
                }
                pixel_studio_pro_v2::Tool::RotateRect => {
                    if let Some(info) = get_rotate_rect_info(action, doc_height) {
                        if info.final_min_x < min_x {
                            min_x = info.final_min_x;
                        }
                        if info.final_min_y < min_y {
                            min_y = info.final_min_y;
                        }
                        if info.final_max_x > max_x {
                            max_x = info.final_max_x;
                        }
                        if info.final_max_y > max_y {
                            max_y = info.final_max_y;
                        }
                    }
                }

                pixel_studio_pro_v2::Tool::PasteImage => {
                    if let Some(meta_str) = &action.meta {
                        if let Ok(meta) = serde_json::from_str::<MetaData>(meta_str) {
                            if let (Some(pixels_b64), Some(rect)) = (&meta.pixels, &meta.rect) {
                                if let Ok(img_data) = b64.decode(pixels_b64) {
                                    if let Ok(img) = image::load_from_memory(&img_data) {
                                        let dst_min_x = rect.from.x;
                                        let dst_max_x = rect.from.x + img.width() as i32;
                                        // Y is inverted (bottom-up in .psp files)
                                        let dst_max_y = doc_height as i32 - rect.from.y;
                                        let dst_min_y = dst_max_y - img.height() as i32;
                                        if dst_min_x < min_x {
                                            min_x = dst_min_x;
                                        }
                                        if dst_min_y < min_y {
                                            min_y = dst_min_y;
                                        }
                                        if dst_max_x > max_x {
                                            max_x = dst_max_x;
                                        }
                                        if dst_max_y > max_y {
                                            max_y = dst_max_y;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                pixel_studio_pro_v2::Tool::Pen
                | pixel_studio_pro_v2::Tool::DotPen
                | pixel_studio_pro_v2::Tool::DitheringPen
                | pixel_studio_pro_v2::Tool::Brush
                | pixel_studio_pro_v2::Tool::OutlineTool
                | pixel_studio_pro_v2::Tool::Eraser
                | pixel_studio_pro_v2::Tool::Fill
                | pixel_studio_pro_v2::Tool::Clear
                | pixel_studio_pro_v2::Tool::EraserPen
                | pixel_studio_pro_v2::Tool::Cut => {
                    update_bounds_from_positions(
                        &action.positions,
                        doc_height,
                        &mut min_x,
                        &mut min_y,
                        &mut max_x,
                        &mut max_y,
                    );
                }
                _ => {}
            }
        }
    }

    (min_x, min_y, max_x, max_y, source_img_opt)
}

fn apply_positions_to_image(
    tool_type: pixel_studio_pro_v2::Tool,
    action: &pixel_studio_pro_v2::Action,
    final_img: &mut RgbaImage,
    min_x: i32,
    min_y: i32,
    img_width: u32,
    img_height: u32,
    doc_height: u32,
    has_data: &mut bool,
) {
    use base64::{Engine as _, engine::general_purpose};
    let pos_bytes = general_purpose::STANDARD
        .decode(&action.positions)
        .unwrap_or_default();
    let col_bytes = general_purpose::STANDARD
        .decode(&action.colors)
        .unwrap_or_default();

    let color = if tool_type == pixel_studio_pro_v2::Tool::Eraser
        || (tool_type == pixel_studio_pro_v2::Tool::Clear && col_bytes.is_empty())
        || tool_type == pixel_studio_pro_v2::Tool::Cut
        || (tool_type == pixel_studio_pro_v2::Tool::EraserPen && col_bytes.is_empty())
    {
        Rgba([0, 0, 0, 0])
    } else if col_bytes.len() >= 4 {
        Rgba([col_bytes[0], col_bytes[1], col_bytes[2], col_bytes[3]])
    } else {
        // For drawing tools that normally have colors, default to transparent if none are provided
        Rgba([0, 0, 0, 0])
    };

    // Only skip processing if the tool required colors but didn't provide any,
    // and we aren't explicitly erasing (which would provide 0 colors anyway).
    if (tool_type == pixel_studio_pro_v2::Tool::Pen
        || tool_type == pixel_studio_pro_v2::Tool::DotPen
        || tool_type == pixel_studio_pro_v2::Tool::DitheringPen
        || tool_type == pixel_studio_pro_v2::Tool::Brush
        || tool_type == pixel_studio_pro_v2::Tool::OutlineTool
        || tool_type == pixel_studio_pro_v2::Tool::Fill)
        && col_bytes.is_empty()
    {
        return;
    }

    for j in (0..pos_bytes.len()).step_by(4) {
        if j + 3 < pos_bytes.len() {
            let px = i16::from_le_bytes([pos_bytes[j], pos_bytes[j + 1]]) as i32 - min_x;
            let py = (doc_height as i32
                - 1
                - i16::from_le_bytes([pos_bytes[j + 2], pos_bytes[j + 3]]) as i32)
                - min_y;

            if px >= 0 && py >= 0 && (px as u32) < img_width && (py as u32) < img_height {
                if tool_type == pixel_studio_pro_v2::Tool::EraserPen {
                    let current_color = *final_img.get_pixel(px as u32, py as u32);
                    if current_color[3] == 0 {
                        final_img.put_pixel(px as u32, py as u32, color);
                    } else {
                        final_img.put_pixel(px as u32, py as u32, Rgba([0, 0, 0, 0]));
                    }
                } else if tool_type == pixel_studio_pro_v2::Tool::Pen
                    || tool_type == pixel_studio_pro_v2::Tool::DotPen
                    || tool_type == pixel_studio_pro_v2::Tool::DitheringPen
                    || tool_type == pixel_studio_pro_v2::Tool::Brush
                    || tool_type == pixel_studio_pro_v2::Tool::OutlineTool
                    || tool_type == pixel_studio_pro_v2::Tool::Eraser
                    || tool_type == pixel_studio_pro_v2::Tool::Clear
                    || tool_type == pixel_studio_pro_v2::Tool::Cut
                {
                    final_img.put_pixel(px as u32, py as u32, color);
                } else if tool_type == pixel_studio_pro_v2::Tool::Fill {
                    flood_fill(final_img, px as u32, py as u32, color);
                }
                *has_data = true;
            }
        }
    }
}

fn apply_move_action(
    action: &pixel_studio_pro_v2::Action,
    final_img: &mut RgbaImage,
    min_x: i32,
    min_y: i32,
    img_width: u32,
    img_height: u32,
    doc_height: u32,
    has_data: &mut bool,
) {
    let pos_bytes = b64.decode(&action.positions).unwrap_or_default();
    if pos_bytes.len() >= 8 {
        let px1 = i16::from_le_bytes([pos_bytes[0], pos_bytes[1]]) as i32;
        let py1 = doc_height as i32 - 1 - i16::from_le_bytes([pos_bytes[2], pos_bytes[3]]) as i32;
        let px2 = i16::from_le_bytes([pos_bytes[4], pos_bytes[5]]) as i32;
        let py2 = doc_height as i32 - 1 - i16::from_le_bytes([pos_bytes[6], pos_bytes[7]]) as i32;
        let dx = px2 - px1;
        let dy = py2 - py1;

        if let Some(meta_str) = &action.meta {
            if let Ok(meta) = serde_json::from_str::<MetaData>(meta_str) {
                if let (Some(from), Some(to)) = (&meta.from, &meta.to) {
                    let sel_min_x = from.x.min(to.x);
                    let sel_max_x = from.x.max(to.x);
                    let sel_min_y = from.y.min(to.y);
                    let sel_max_y = from.y.max(to.y);

                    let top_down_min_y = doc_height as i32 - 1 - sel_max_y;
                    let top_down_max_y = doc_height as i32 - 1 - sel_min_y;

                    let mut moved_pixels = Vec::new();

                    for y in top_down_min_y..=top_down_max_y {
                        for x in sel_min_x..=sel_max_x {
                            let canvas_x = x - min_x;
                            let canvas_y = y - min_y;

                            if canvas_x >= 0
                                && canvas_y >= 0
                                && (canvas_x as u32) < img_width
                                && (canvas_y as u32) < img_height
                            {
                                let p = *final_img.get_pixel(canvas_x as u32, canvas_y as u32);
                                moved_pixels.push((x, y, p));
                                final_img.put_pixel(
                                    canvas_x as u32,
                                    canvas_y as u32,
                                    Rgba([0, 0, 0, 0]),
                                );
                                *has_data = true;
                            }
                        }
                    }

                    for (x, y, p) in moved_pixels {
                        let shifted_x = x + dx - min_x;
                        let shifted_y = y + dy - min_y;

                        if shifted_x >= 0
                            && shifted_y >= 0
                            && (shifted_x as u32) < img_width
                            && (shifted_y as u32) < img_height
                        {
                            final_img.put_pixel(shifted_x as u32, shifted_y as u32, p);
                            *has_data = true;
                        }
                    }
                }
            }
        } else {
            let mut moved_pixels = Vec::new();
            for j in (8..pos_bytes.len()).step_by(4) {
                if j + 3 < pos_bytes.len() {
                    let px = i16::from_le_bytes([pos_bytes[j], pos_bytes[j + 1]]) as i32;
                    let py = doc_height as i32 - 1 - i16::from_le_bytes([pos_bytes[j + 2], pos_bytes[j + 3]]) as i32;

                    let canvas_x = px - min_x;
                    let canvas_y = py - min_y;
                    if canvas_x >= 0 && canvas_y >= 0 && (canvas_x as u32) < img_width && (canvas_y as u32) < img_height {
                        let p = *final_img.get_pixel(canvas_x as u32, canvas_y as u32);
                        moved_pixels.push((px, py, p));
                    }
                }
            }

            for (px, py, _) in &moved_pixels {
                let canvas_x = px - min_x;
                let canvas_y = py - min_y;
                final_img.put_pixel(canvas_x as u32, canvas_y as u32, Rgba([0, 0, 0, 0]));
                *has_data = true;
            }

            for (px, py, p) in moved_pixels {
                let shifted_x = px + dx - min_x;
                let shifted_y = py + dy - min_y;

                if shifted_x >= 0 && shifted_y >= 0 && (shifted_x as u32) < img_width && (shifted_y as u32) < img_height {
                    final_img.put_pixel(shifted_x as u32, shifted_y as u32, p);
                    *has_data = true;
                }
            }
        }
    }
}

fn apply_paste_import_action(
    tool_type: pixel_studio_pro_v2::Tool,
    action: &pixel_studio_pro_v2::Action,
    final_img: &mut RgbaImage,
    min_x: i32,
    min_y: i32,
    img_width: u32,
    img_height: u32,
    doc_height: u32,
    has_data: &mut bool,
) {
    if let Some(meta_str) = &action.meta {
        if let Ok(meta) = serde_json::from_str::<MetaData>(meta_str) {
            if let (Some(pixels_b64), Some(rect)) = (&meta.pixels, &meta.rect) {
                if let Ok(img_data) = b64.decode(pixels_b64) {
                    if let Ok(img) = image::load_from_memory(&img_data) {
                        let rgba_patch = img.to_rgba8();
                        let start_x = rect.from.x - min_x;
                        let start_y =
                            (doc_height as i32 - rect.from.y - rgba_patch.height() as i32) - min_y;

                        for y in 0..rgba_patch.height() {
                            for x in 0..rgba_patch.width() {
                                let dst_x = start_x + (x as i32);
                                let dst_y = start_y + (y as i32);

                                if dst_x >= 0
                                    && dst_y >= 0
                                    && (dst_x as u32) < img_width
                                    && (dst_y as u32) < img_height
                                {
                                    let p = rgba_patch.get_pixel(x, y);

                                    if tool_type == pixel_studio_pro_v2::Tool::PasteImage {
                                        if p[3] > 0 {
                                            use image::Pixel;
                                            let mut bg_p =
                                                *final_img.get_pixel(dst_x as u32, dst_y as u32);
                                            bg_p.blend(p);
                                            final_img.put_pixel(dst_x as u32, dst_y as u32, bg_p);
                                            *has_data = true;
                                        }
                                    } else {
                                        if p[3] == 0 {
                                            final_img.put_pixel(
                                                dst_x as u32,
                                                dst_y as u32,
                                                Rgba([0, 0, 0, 0]),
                                            );
                                            *has_data = true;
                                        } else {
                                            final_img.put_pixel(dst_x as u32, dst_y as u32, *p);
                                            *has_data = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn apply_transform_action(
    tool_type: pixel_studio_pro_v2::Tool,
    action: &pixel_studio_pro_v2::Action,
    final_img: &mut RgbaImage,
    min_x: i32,
    min_y: i32,
    img_width: u32,
    img_height: u32,
    doc_height: u32,
    has_data: &mut bool,
) {
    if let Some(meta_str) = &action.meta {
        if let Ok(meta) = serde_json::from_str::<MetaData>(meta_str) {
            if let Some(rect) = &meta.rect {
                let sel_min_x = rect.from.x - min_x;
                let sel_max_x = rect
                    .to
                    .as_ref()
                    .map_or(sel_min_x, |to| to.x - min_x)
                    .max(sel_min_x + rect.width.unwrap_or(0) - 1);
                let mut sel_min_y = (doc_height as i32 - 1 - rect.from.y) - min_y;
                let mut sel_max_y =
                    (doc_height as i32 - 1 - rect.to.as_ref().map_or(rect.from.y, |to| to.y))
                        - min_y;

                if sel_min_y > sel_max_y {
                    std::mem::swap(&mut sel_min_y, &mut sel_max_y);
                }
                sel_max_y = sel_max_y.max(sel_min_y + rect.height.unwrap_or(0) - 1);

                let sel_w = sel_max_x - sel_min_x + 1;
                let sel_h = sel_max_y - sel_min_y + 1;

                let temp_img = final_img.clone();

                if tool_type == pixel_studio_pro_v2::Tool::RotateLeft
                    || tool_type == pixel_studio_pro_v2::Tool::RotateRight
                    || tool_type == pixel_studio_pro_v2::Tool::FlipByX
                    || tool_type == pixel_studio_pro_v2::Tool::FlipByY
                {
                    for y in 0..sel_h {
                        for x in 0..sel_w {
                            let src_x = sel_min_x + x;
                            let src_y = sel_min_y + y;
                            if src_x >= 0
                                && src_y >= 0
                                && (src_x as u32) < img_width
                                && (src_y as u32) < img_height
                            {
                                final_img.put_pixel(src_x as u32, src_y as u32, Rgba([0, 0, 0, 0]));
                            }
                        }
                    }
                }

                for y in 0..sel_h {
                    for x in 0..sel_w {
                        let src_x = sel_min_x + x;
                        let src_y = sel_min_y + y;
                        if src_x >= 0
                            && src_y >= 0
                            && (src_x as u32) < img_width
                            && (src_y as u32) < img_height
                        {
                            let color = *temp_img.get_pixel(src_x as u32, src_y as u32);
                            if color[3] > 0 {
                                let (dst_x, dst_y) = match tool_type {
                                    pixel_studio_pro_v2::Tool::MirrorByX => {
                                        (sel_min_x + (sel_w - 1 - x), sel_min_y + y)
                                    }
                                    pixel_studio_pro_v2::Tool::MirrorByY => {
                                        (sel_min_x + x, sel_min_y + (sel_h - 1 - y))
                                    }
                                    pixel_studio_pro_v2::Tool::FlipByX => {
                                        (sel_min_x + (sel_w - 1 - x), sel_min_y + y)
                                    }
                                    pixel_studio_pro_v2::Tool::FlipByY => {
                                        (sel_min_x + x, sel_min_y + (sel_h - 1 - y))
                                    }
                                    pixel_studio_pro_v2::Tool::RotateRight => {
                                        let new_x = y;
                                        let new_y = sel_w - 1 - x;
                                        (
                                            new_x + sel_min_x + (sel_w - sel_h) / 2,
                                            new_y + sel_min_y + (sel_h - sel_w) / 2,
                                        )
                                    }
                                    pixel_studio_pro_v2::Tool::RotateLeft => {
                                        let new_x = sel_h - 1 - y;
                                        let new_y = x;
                                        (
                                            new_x + sel_min_x + (sel_w - sel_h) / 2,
                                            new_y + sel_min_y + (sel_h - sel_w) / 2,
                                        )
                                    }
                                    _ => (src_x, src_y),
                                };

                                if dst_x >= 0
                                    && dst_y >= 0
                                    && (dst_x as u32) < img_width
                                    && (dst_y as u32) < img_height
                                {
                                    final_img.put_pixel(dst_x as u32, dst_y as u32, color);
                                    *has_data = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

struct RotateRectInfo {
    rect_min_x: i32,

    rect_min_y: i32,

    w: i32,
    h: i32,
    px3: i32,
    py3: i32,

    final_min_x: i32,
    final_min_y: i32,
    final_max_x: i32,
    final_max_y: i32,
    cos_a: f32,
    sin_a: f32,
    cx: f32,
    cy: f32,
    min_rx: f32,
    min_ry: f32,
    max_rx: f32,
    max_ry: f32,
}

fn get_rotate_rect_info(
    action: &pixel_studio_pro_v2::Action,
    doc_height: u32,
) -> Option<RotateRectInfo> {
    use base64::{Engine as _, engine::general_purpose};
    let pos_bytes = general_purpose::STANDARD
        .decode(&action.positions)
        .unwrap_or_default();

    if pos_bytes.len() < 12 {
        return None;
    }

    let px1 = i16::from_le_bytes([pos_bytes[0], pos_bytes[1]]) as i32;
    let py1 = doc_height as i32 - 1 - i16::from_le_bytes([pos_bytes[2], pos_bytes[3]]) as i32;
    let px2 = i16::from_le_bytes([pos_bytes[4], pos_bytes[5]]) as i32;
    let py2 = doc_height as i32 - 1 - i16::from_le_bytes([pos_bytes[6], pos_bytes[7]]) as i32;
    let px3 = i16::from_le_bytes([pos_bytes[8], pos_bytes[9]]) as i32;
    let py3 = doc_height as i32 - 1 - i16::from_le_bytes([pos_bytes[10], pos_bytes[11]]) as i32;

    let rect_min_x = px1.min(px2);
    let rect_max_x = px1.max(px2);
    let rect_min_y = py1.min(py2);
    let rect_max_y = py1.max(py2);

    let w = rect_max_x - rect_min_x + 1;
    let h = rect_max_y - rect_min_y + 1;

    // Bounds validation to prevent OOM
    if w > 8192 || h > 8192 || w <= 0 || h <= 0 {
        return None;
    }

    // Try parsing the angle
    // Meta="0" means 0 degrees in string format. Sometimes it might be JSON in the future, so try both.
    let angle_deg: f32 = action
        .meta
        .as_deref()
        .and_then(|m| m.parse().ok())
        .or_else(|| {
            action.meta.as_deref().and_then(|m| {
                serde_json::from_str::<serde_json::Value>(m)
                    .ok()
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
            })
        })
        .unwrap_or(0.0);

    let angle_rad = angle_deg * std::f32::consts::PI / 180.0;
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();

    let cx = (w as f32 - 1.0) / 2.0;
    let cy = (h as f32 - 1.0) / 2.0;

    let mut min_rx = f32::MAX;
    let mut min_ry = f32::MAX;
    let mut max_rx = f32::MIN;
    let mut max_ry = f32::MIN;

    let corners = [
        (0.0, 0.0),
        (w as f32 - 1.0, 0.0),
        (0.0, h as f32 - 1.0),
        (w as f32 - 1.0, h as f32 - 1.0),
    ];

    for &(x, y) in &corners {
        let rel_x = x - cx;
        let rel_y = y - cy;
        let rot_x = cx + rel_x * cos_a - rel_y * sin_a;
        let rot_y = cy + rel_x * sin_a + rel_y * cos_a;
        if rot_x < min_rx {
            min_rx = rot_x;
        }
        if rot_y < min_ry {
            min_ry = rot_y;
        }
        if rot_x > max_rx {
            max_rx = rot_x;
        }
        if rot_y > max_ry {
            max_ry = rot_y;
        }
    }

    let offset_x = px3 - rect_min_x;
    let offset_y = py3 - rect_min_y;

    let final_min_x = min_rx.floor() as i32 + rect_min_x + offset_x;
    let final_min_y = min_ry.floor() as i32 + rect_min_y + offset_y;
    let final_max_x = max_rx.ceil() as i32 + rect_min_x + offset_x;
    let final_max_y = max_ry.ceil() as i32 + rect_min_y + offset_y;

    Some(RotateRectInfo {
        rect_min_x,
        rect_min_y,
        w,
        h,
        px3,
        py3,
        final_min_x,
        final_min_y,
        final_max_x,
        final_max_y,
        cos_a,
        sin_a,
        cx,
        cy,
        min_rx,
        min_ry,
        max_rx,
        max_ry,
    })
}

fn apply_rotate_rect_action(
    action: &pixel_studio_pro_v2::Action,
    final_img: &mut RgbaImage,
    min_x: i32,
    min_y: i32,
    img_width: u32,
    img_height: u32,
    doc_height: u32,
    has_data: &mut bool,
) {
    let Some(info) = get_rotate_rect_info(action, doc_height) else {
        return;
    };

    use base64::{Engine as _, engine::general_purpose};
    let pos_bytes = general_purpose::STANDARD
        .decode(&action.positions)
        .unwrap_or_default();

    // Extract source pixels
    let mut extracted_pixels = vec![Rgba([0, 0, 0, 0]); (info.w * info.h) as usize];

    let count = pos_bytes.len() / 4;

    if count == 3 {
        for y in 0..info.h {
            for x in 0..info.w {
                let src_x = info.rect_min_x + x - min_x;
                let src_y = info.rect_min_y + y - min_y;

                if src_x >= 0
                    && src_y >= 0
                    && (src_x as u32) < img_width
                    && (src_y as u32) < img_height
                {
                    let p = *final_img.get_pixel(src_x as u32, src_y as u32);
                    extracted_pixels[(x + y * info.w) as usize] = p;
                    final_img.put_pixel(src_x as u32, src_y as u32, Rgba([0, 0, 0, 0]));
                }
            }
        }
    } else {
        for i in 3..count {
            let idx = i * 4;
            let px = i16::from_le_bytes([pos_bytes[idx], pos_bytes[idx + 1]]) as i32;
            let py = doc_height as i32
                - 1
                - i16::from_le_bytes([pos_bytes[idx + 2], pos_bytes[idx + 3]]) as i32;

            let rel_x = px - info.rect_min_x;
            let rel_y = py - info.rect_min_y;

            let src_x = px - min_x;
            let src_y = py - min_y;

            if src_x >= 0
                && src_y >= 0
                && (src_x as u32) < img_width
                && (src_y as u32) < img_height
                && rel_x >= 0
                && rel_x < info.w
                && rel_y >= 0
                && rel_y < info.h
            {
                let p = *final_img.get_pixel(src_x as u32, src_y as u32);
                extracted_pixels[(rel_x + rel_y * info.w) as usize] = p;
                final_img.put_pixel(src_x as u32, src_y as u32, Rgba([0, 0, 0, 0]));
            }
        }
    }

    let rot_w = (info.max_rx - info.min_rx).round() as i32 + 1;
    let rot_h = (info.max_ry - info.min_ry).round() as i32 + 1;

    // Nearest neighbor rotation
    // We map destination pixels back to source pixels
    for ry in 0..rot_h {
        for rx in 0..rot_w {
            let dst_cx = info.min_rx.floor() + rx as f32;
            let dst_cy = info.min_ry.floor() + ry as f32;

            let rel_x = dst_cx - info.cx;
            let rel_y = dst_cy - info.cy;

            // inverse rotation
            let src_x = info.cx + rel_x * info.cos_a + rel_y * info.sin_a;
            let src_y = info.cy - rel_x * info.sin_a + rel_y * info.cos_a;

            let src_x_i = src_x.round() as i32;
            let src_y_i = src_y.round() as i32;

            if src_x_i >= 0 && src_x_i < info.w && src_y_i >= 0 && src_y_i < info.h {
                let p = extracted_pixels[(src_x_i + src_y_i * info.w) as usize];
                if p[3] != 0 {
                    let final_x = info.px3 + rx + info.min_rx.floor() as i32 - min_x;
                    let final_y = info.py3 + ry + info.min_ry.floor() as i32 - min_y;

                    if final_x >= 0
                        && final_y >= 0
                        && (final_x as u32) < img_width
                        && (final_y as u32) < img_height
                    {
                        final_img.put_pixel(final_x as u32, final_y as u32, p);
                        *has_data = true;
                    }
                }
            }
        }
    }
}

fn apply_replace_color_action(
    action: &pixel_studio_pro_v2::Action,
    final_img: &mut RgbaImage,
    min_x: i32,
    min_y: i32,
    img_width: u32,
    img_height: u32,
    doc_height: u32,
) {
    use base64::{Engine as _, engine::general_purpose};
    let pos_bytes = general_purpose::STANDARD
        .decode(&action.positions)
        .unwrap_or_default();
    let col_bytes = general_purpose::STANDARD
        .decode(&action.colors)
        .unwrap_or_default();
    if pos_bytes.len() >= 4 && col_bytes.len() >= 4 {
        let target_x = i16::from_le_bytes([pos_bytes[0], pos_bytes[1]]) as i32 - min_x;
        let target_y =
            (doc_height as i32 - 1 - i16::from_le_bytes([pos_bytes[2], pos_bytes[3]]) as i32)
                - min_y;
        if target_x >= 0
            && target_y >= 0
            && (target_x as u32) < img_width
            && (target_y as u32) < img_height
        {
            let target_color = *final_img.get_pixel(target_x as u32, target_y as u32);
            let new_color = Rgba([col_bytes[0], col_bytes[1], col_bytes[2], col_bytes[3]]);

            for y in 0..img_height {
                for x in 0..img_width {
                    if *final_img.get_pixel(x, y) == target_color {
                        final_img.put_pixel(x, y, new_color);
                    }
                }
            }
        }
    }
}

fn replay_actions(
    history: &History,
    min_x: i32,
    min_y: i32,
    img_width: u32,
    img_height: u32,
    doc_height: u32,
    source_img_opt: Option<RgbaImage>,
) -> (RgbaImage, bool) {
    let mut final_img = RgbaImage::new(img_width, img_height);
    let mut has_data = false;

    let mut source_has_pixels = false;
    // Draw source image first if available
    if let Some(src_img) = source_img_opt {
        let offset_x = -min_x;
        let offset_y = -min_y;
        for y in 0..src_img.height() {
            for x in 0..src_img.width() {
                let p = *src_img.get_pixel(x, y);
                if p[3] > 0 {
                    source_has_pixels = true;
                }

                let dst_x = offset_x + x as i32;
                let dst_y = offset_y + y as i32;
                if dst_x >= 0
                    && dst_y >= 0
                    && (dst_x as u32) < img_width
                    && (dst_y as u32) < img_height
                {
                    final_img.put_pixel(dst_x as u32, dst_y as u32, p);
                    has_data = true;
                }
            }
        }
    }

    // Second pass: replay actions onto the sized canvas if there was no valid source image
    if !source_has_pixels {
        // Actions must be replayed up to index. Actually, history.index can sometimes point past the end.
        let replay_count = std::cmp::min(history.index as usize, history.actions.len());
        for action in history.actions.iter().take(replay_count) {
            if let Ok(tool_type) = pixel_studio_pro_v2::Tool::try_from(action.tool) {
                match tool_type {
                    pixel_studio_pro_v2::Tool::Move => {
                        apply_move_action(
                            action,
                            &mut final_img,
                            min_x,
                            min_y,
                            img_width,
                            img_height,
                            doc_height,
                            &mut has_data,
                        );
                    }
                    pixel_studio_pro_v2::Tool::RotateRect => {
                        apply_rotate_rect_action(
                            action,
                            &mut final_img,
                            min_x,
                            min_y,
                            img_width,
                            img_height,
                            doc_height,
                            &mut has_data,
                        );
                    }
                    pixel_studio_pro_v2::Tool::PasteImage => {
                        apply_paste_import_action(
                            tool_type,
                            action,
                            &mut final_img,
                            min_x,
                            min_y,
                            img_width,
                            img_height,
                            doc_height,
                            &mut has_data,
                        );
                    }
                    pixel_studio_pro_v2::Tool::Pen
                    | pixel_studio_pro_v2::Tool::DotPen
                    | pixel_studio_pro_v2::Tool::DitheringPen
                    | pixel_studio_pro_v2::Tool::Brush
                    | pixel_studio_pro_v2::Tool::OutlineTool
                    | pixel_studio_pro_v2::Tool::Fill
                    | pixel_studio_pro_v2::Tool::Eraser
                    | pixel_studio_pro_v2::Tool::Clear
                    | pixel_studio_pro_v2::Tool::EraserPen
                    | pixel_studio_pro_v2::Tool::Cut => {
                        apply_positions_to_image(
                            tool_type,
                            action,
                            &mut final_img,
                            min_x,
                            min_y,
                            img_width,
                            img_height,
                            doc_height,
                            &mut has_data,
                        );
                    }
                    pixel_studio_pro_v2::Tool::MirrorByX
                    | pixel_studio_pro_v2::Tool::MirrorByY
                    | pixel_studio_pro_v2::Tool::FlipByX
                    | pixel_studio_pro_v2::Tool::FlipByY
                    | pixel_studio_pro_v2::Tool::RotateLeft
                    | pixel_studio_pro_v2::Tool::RotateRight => {
                        apply_transform_action(
                            tool_type,
                            action,
                            &mut final_img,
                            min_x,
                            min_y,
                            img_width,
                            img_height,
                            doc_height,
                            &mut has_data,
                        );
                    }
                    pixel_studio_pro_v2::Tool::ReplaceColor => {
                        apply_replace_color_action(
                            action,
                            &mut final_img,
                            min_x,
                            min_y,
                            img_width,
                            img_height,
                            doc_height,
                        );
                    }
                    pixel_studio_pro_v2::Tool::Pipette
                    | pixel_studio_pro_v2::Tool::MoveCamera
                    | pixel_studio_pro_v2::Tool::GenericTool
                    | pixel_studio_pro_v2::Tool::Copy
                    | pixel_studio_pro_v2::Tool::Paste
                    | pixel_studio_pro_v2::Tool::MagicWand
                    | pixel_studio_pro_v2::Tool::ColorAdjustment
                    | pixel_studio_pro_v2::Tool::PixelSelect
                    | pixel_studio_pro_v2::Tool::Lasso
                    | pixel_studio_pro_v2::Tool::Cursor => {
                        // Ignore these UI/selection tools or unknown tools
                    }
                }
            }
        }
    }

    (final_img, has_data)
}

pub fn convert(doc: pixel_studio_pro_v2::Document) -> Result<Document> {
    let mut layers: Vec<Layer> = Vec::new();
    let mut frames: Vec<Frame> = Vec::new();
    let mut cels: Vec<Cel> = Vec::new();
    let mut images: Vec<Image> = Vec::new();

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
                    let prev_img_idx = cels[last_cel_idx].image_index;

                    let new_cel = Cel {
                        frame_index,
                        layer_index,
                        x: prev_x,
                        y: prev_y,
                        image_index: prev_img_idx,
                    };
                    last_cel_per_layer[layer_index] = Some(cels.len());
                    cels.push(new_cel);
                }
            } else if let Some(history_str) = &psp_layer.history_json {
                let history = serde_json::from_str::<History>(history_str).map_err(|e| {
                    anyhow!(
                        "Failed to parse history JSON for layer {}: {}",
                        layer_index,
                        e
                    )
                })?;

                let (min_x, min_y, max_x, max_y, source_img_opt) =
                    calculate_bounds(&history, doc_width, doc_height);

                // Cap max dimensions to a reasonable safeguard (e.g. 4096) to avoid OOM panics
                let img_width = (max_x - min_x).clamp(1, 4096) as u32;
                let img_height = (max_y - min_y).clamp(1, 4096) as u32;

                let (final_img, has_data) = replay_actions(
                    &history,
                    min_x,
                    min_y,
                    img_width,
                    img_height,
                    doc_height,
                    source_img_opt,
                );

                if has_data {
                    let image_index = images.len();
                    images.push(Image {
                        width: u16::try_from(img_width).unwrap_or(u16::MAX),
                        height: u16::try_from(img_height).unwrap_or(u16::MAX),
                        rgba: final_img.into_raw(),
                    });

                    let cel = Cel {
                        frame_index,
                        layer_index,
                        x: (psp_layer.sx + min_x).clamp(i16::MIN as i32, i16::MAX as i32) as i16,
                        y: (psp_layer.sy + min_y).clamp(i16::MIN as i32, i16::MAX as i32) as i16,
                        image_index,
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
        images,
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
