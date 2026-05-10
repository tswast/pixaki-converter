import re

with open("crates/pixel-studio-pro-v2-converter/src/lib.rs", "r") as f:
    content = f.read()

content = content.replace(
"""fn apply_paste_import_action(
    tool_type: pixel_studio_pro_v2::Tool,
    action: &pixel_studio_pro_v2::Action,
    final_img: &mut RgbaImage,
    min_x: i32,
    min_y: i32,
    img_width: u32,
    img_height: u32,
    _doc_height: u32,
    has_data: &mut bool,
)""",
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
)""")

content = content.replace(
"""                        let _ = doc_height; // prevent warning
                        let rgba_patch = img.to_rgba8();
                        let start_x = rect.from.x - min_x;
                        let start_y = rect.from.y - min_y;""",
"""                        let rgba_patch = img.to_rgba8();
                        let start_x = rect.from.x - min_x;
                        let start_y = (doc_height as i32 - rect.from.y - rgba_patch.height() as i32) - min_y;""")


with open("crates/pixel-studio-pro-v2-converter/src/lib.rs", "w") as f:
    f.write(content)
