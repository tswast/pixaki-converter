use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Document {
    pub version: u32,
    pub id: String,
    pub name: String,
    pub source: Option<String>,
    pub width: u16,
    pub height: u16,
    #[serde(rename = "Type")]
    pub doc_type: u32,
    pub clips: Vec<Clip>,
    pub background: bool,
    pub background_color: Option<Color>,
    pub tile_mode: bool,
    pub tile_fade: u32,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Clip {
    pub id: String,
    pub name: String,
    pub frames: Vec<Frame>,
    pub layer_types: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Frame {
    pub id: String,
    pub delay: f64,
    pub layers: Vec<Layer>,
    pub layer_groups: Vec<LayerGroup>,
    pub active_layer_index: Option<u32>,
    #[serde(rename = "_reference")]
    pub reference: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Layer {
    pub id: String,
    pub name: String,
    pub opacity: f64,
    pub transparency: f64,
    pub hidden: bool,
    pub linked: bool,
    pub outline: u32,
    pub lock: u32,
    pub sx: i32,
    pub sy: i32,
    pub version: u32,
    #[serde(rename = "_historyJson")]
    pub history_json: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct LayerGroup {
    pub id: String,
    pub name: String,
    pub index: u32,
    pub hidden: bool,
    pub collapsed: bool,
    pub layers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct History {
    pub actions: Vec<Action>,
    pub index: u32,
    #[serde(rename = "_source")]
    pub source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Action {
    pub tool: u32,
    pub color_indexes: Option<Vec<u32>>,
    pub positions: String,
    pub colors: String,
    pub meta: Option<String>,
    pub invalid: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_psp_v2_deserialization() {
        let json_data = r#"{
            "Version": 2,
            "Id": "abc",
            "Name": "test",
            "Width": 32,
            "Height": 32,
            "Type": 0,
            "Clips": [
                {
                    "Id": "clip1",
                    "Name": "Clip 1",
                    "Frames": [
                        {
                            "Id": "frame1",
                            "Delay": 0.5,
                            "Layers": [
                                {
                                    "Id": "layer1",
                                    "Name": "Layer 1",
                                    "Opacity": 0.8,
                                    "Transparency": -1.0,
                                    "Hidden": false,
                                    "Linked": false,
                                    "Outline": 0,
                                    "Lock": 0,
                                    "Sx": 0,
                                    "Sy": 0,
                                    "Version": 1,
                                    "_historyJson": "{\"Actions\":[],\"Index\":0}"
                                }
                            ],
                            "LayerGroups": []
                        }
                    ],
                    "LayerTypes": [0]
                }
            ],
            "Background": false,
            "TileMode": false,
            "TileFade": 0
        }"#;

        let doc: Document = serde_json::from_str(json_data).unwrap();
        assert_eq!(doc.width, 32);
        assert_eq!(doc.clips[0].frames[0].layers[0].opacity, 0.8);
    }
}
