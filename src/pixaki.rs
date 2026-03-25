use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub sprites: Vec<Sprite>,
}

#[derive(Debug, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sprite {
    pub layers: Vec<Layer>,
    pub cels: Vec<Cel>,
    pub size: Size,
    pub duration: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Layer {
    pub name: String,
    #[serde(rename = "isVisible")]
    pub is_visible: bool,
    pub opacity: f64,
    #[serde(rename = "blendMode")]
    pub blend_mode: String,
    pub clips: Vec<Clip>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub item_identifier: String,
    pub range: Option<Range>,
}

#[derive(Debug, Deserialize)]
pub struct Range {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cel {
    pub identifier: String,
    pub frame: Vec<Vec<f64>>,
}
