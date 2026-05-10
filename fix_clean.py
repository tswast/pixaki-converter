import re

with open("crates/pixel-studio-pro-v2-converter/src/lib.rs", "r") as f:
    content = f.read()

# 1. RotateRect offset logic fix
content = content.replace(
"""    let offset_x = px3 - rect_min_x;
    let offset_y = py3 - rect_min_y;""",
"""    let offset_x = px3;
    let offset_y = -py3;""")

# 2. PasteImage coordinate inversion logic removal for `calculate_bounds`
content = content.replace(
"""            let dst_min_x = rect.from.x;
            let dst_max_x = rect.from.x + img.width() as i32;
            // Y is inverted (bottom-up in .psp files)
            let dst_max_y = doc_height as i32 - rect.from.y;
            let dst_min_y = dst_max_y - img.height() as i32;""",
"""            let dst_min_x = rect.from.x;
            let dst_max_x = rect.from.x + img.width() as i32;
            let dst_min_y = rect.from.y;
            let dst_max_y = dst_min_y + img.height() as i32;""")

# 3. PasteImage coordinate inversion logic removal for `apply_paste_import_action`
content = content.replace(
"""fn apply_paste_import_action(
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
                            (doc_height as i32 - rect.from.y - rgba_patch.height() as i32) - min_y;""",
"""#[allow(unused_variables)]
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
                        let start_y = rect.from.y - min_y;""")

# 4. Move tool calculate bounds fix
content = content.replace(
"""                pixel_studio_pro_v2::Tool::Move => {
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
                                    let shifted_max_y = top_down_max_y + dy;""",
"""                pixel_studio_pro_v2::Tool::Move => {
                    let pos_bytes = b64.decode(&action.positions).unwrap_or_default();
                    if pos_bytes.len() >= 8 {
                        let px1 = i16::from_le_bytes([pos_bytes[0], pos_bytes[1]]) as i32;
                        let py1 = i16::from_le_bytes([pos_bytes[2], pos_bytes[3]]) as i32;
                        let px2 = i16::from_le_bytes([pos_bytes[4], pos_bytes[5]]) as i32;
                        let py2 = i16::from_le_bytes([pos_bytes[6], pos_bytes[7]]) as i32;
                        let dx = px2 - px1;
                        let dy = py2 - py1;

                        if let Some(meta_str) = &action.meta {
                            if let Ok(meta) = serde_json::from_str::<MetaData>(meta_str) {
                                if let (Some(from), Some(to)) = (&meta.from, &meta.to) {
                                    let sel_min_x = from.x.min(to.x);
                                    let sel_max_x = from.x.max(to.x);
                                    let sel_min_y = from.y.min(to.y);
                                    let sel_max_y = from.y.max(to.y);

                                    let shifted_min_x = sel_min_x + dx;
                                    let shifted_max_x = sel_max_x + dx;
                                    let shifted_min_y = sel_min_y - dy;
                                    let shifted_max_y = sel_max_y - dy;""")

# 5. Move tool application logic fix
content = content.replace(
"""                    pixel_studio_pro_v2::Tool::Move => {
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

                                        let mut new_img = final_img.clone();

                                        // 1. Clear original area
                                        for y in top_down_min_y..=top_down_max_y {
                                            for x in sel_min_x..=sel_max_x {""",
"""                    pixel_studio_pro_v2::Tool::Move => {
                        let pos_bytes = b64.decode(&action.positions).unwrap_or_default();
                        if pos_bytes.len() >= 8 {
                            let px1 = i16::from_le_bytes([pos_bytes[0], pos_bytes[1]]) as i32;
                            let py1 = i16::from_le_bytes([pos_bytes[2], pos_bytes[3]]) as i32;
                            let px2 = i16::from_le_bytes([pos_bytes[4], pos_bytes[5]]) as i32;
                            let py2 = i16::from_le_bytes([pos_bytes[6], pos_bytes[7]]) as i32;
                            let dx = px2 - px1;
                            let dy = py2 - py1;

                            if let Some(meta_str) = &action.meta {
                                if let Ok(meta) = serde_json::from_str::<MetaData>(meta_str) {
                                    if let (Some(from), Some(to)) = (&meta.from, &meta.to) {
                                        let sel_min_x = from.x.min(to.x);
                                        let sel_max_x = from.x.max(to.x);
                                        let sel_min_y = from.y.min(to.y);
                                        let sel_max_y = from.y.max(to.y);

                                        let mut new_img = final_img.clone();

                                        // 1. Clear original area
                                        for y in sel_min_y..=sel_max_y {
                                            for x in sel_min_x..=sel_max_x {""")

content = content.replace(
"""                                        // 2. Draw to new area
                                        for y in top_down_min_y..=top_down_max_y {
                                            for x in sel_min_x..=sel_max_x {
                                                if let Some(px) = get_pixel_safe(
                                                    final_img,
                                                    x,
                                                    y,
                                                    min_x,
                                                    min_y,
                                                    img_width,
                                                    img_height,
                                                ) {
                                                    if px[3] > 0 {
                                                        put_pixel_safe(
                                                            &mut new_img,
                                                            x + dx,
                                                            y + dy,
                                                            min_x,
                                                            min_y,
                                                            img_width,
                                                            img_height,
                                                            px,
                                                        );""",
"""                                        // 2. Draw to new area
                                        for y in sel_min_y..=sel_max_y {
                                            for x in sel_min_x..=sel_max_x {
                                                if let Some(px) = get_pixel_safe(
                                                    final_img,
                                                    x,
                                                    y,
                                                    min_x,
                                                    min_y,
                                                    img_width,
                                                    img_height,
                                                ) {
                                                    if px[3] > 0 {
                                                        put_pixel_safe(
                                                            &mut new_img,
                                                            x + dx,
                                                            y - dy,
                                                            min_x,
                                                            min_y,
                                                            img_width,
                                                            img_height,
                                                            px,
                                                        );""")

with open("crates/pixel-studio-pro-v2-converter/src/lib.rs", "w") as f:
    f.write(content)
