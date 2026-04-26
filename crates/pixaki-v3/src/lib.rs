use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub sprites: Vec<Sprite>,
    pub palette: Option<Palette>,
    pub grid_settings: Option<GridSettings>,
    pub animation_speed: Option<f64>,
    pub primary_sprite_identifier: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Palette {
    pub name: Option<String>,
    pub identifier: Option<String>,
    pub colors: Vec<Color>,
    pub selected_color_index: Option<usize>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Color {
    pub hue: f64,
    pub saturation: f64,
    pub brightness: f64,
    pub alpha: f64,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GridSettings {
    pub color: Option<Color>,
    pub size: Option<[u32; 2]>,
    pub show_grid: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Sprite {
    pub layers: Vec<Layer>,
    pub cels: Vec<Cel>,
    pub size: Size,
    pub duration: u32,
    pub identifier: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Layer {
    pub name: String,
    #[serde(rename = "isVisible")]
    pub is_visible: bool,
    pub opacity: f64,
    #[serde(rename = "blendMode", default)]
    pub blend_mode: Option<String>,
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    pub identifier: Option<String>,
    pub clips: Vec<Clip>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub item_identifier: String,
    pub range: Option<Range>,
    pub identifier: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Range {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Cel {
    pub identifier: String,
    pub frame: Vec<Vec<f64>>,
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    pub opacity: Option<f64>,
    #[serde(rename = "isVisible")]
    pub is_visible: Option<bool>,
    pub requires_trim: Option<bool>,
    pub container_size: Option<[f64; 2]>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v3_deserialization() {
        let json_data = r#"
        {
          "sprites": [
            {
              "size": { "width": 32, "height": 32 },
              "duration": 1,
              "layers": [
                {
                  "name": "Layer 1",
                  "isVisible": true,
                  "opacity": 1.0,
                  "clips": []
                }
              ],
              "cels": []
            }
          ]
        }
        "#;
        let doc: Document = serde_json::from_str(json_data).unwrap();
        assert_eq!(doc.sprites[0].size.width, 32.0);
        assert_eq!(doc.sprites[0].layers[0].name, "Layer 1");
    }
}
