use clap::Parser;
use pixaki_converter::aseprite::{
    AsepriteHeader, BlendMode, CelChunk, CelType, Chunk, ChunkType, FrameHeader, LayerChunk,
    LayerFlags, LayerType,
};
use pixaki_converter::pixaki::Document;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Seek, SeekFrom};

/// A simple program to convert Pixaki files to Aseprite files.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to the .pixaki directory
    pixaki_path: std::path::PathBuf,

    /// The path to the output .aseprite file
    aseprite_path: std::path::PathBuf,
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

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    // 1. Read and parse document.json
    let document_path = cli.pixaki_path.join("document.json");
    let json_str = fs::read_to_string(document_path)?;
    let document: Document =
        serde_json::from_str(&json_str).expect("Unable to parse document.json");

    // 2. Extract data
    let sprite = document.sprites.get(0).expect("No sprite found");
    let width = sprite.size.width as u16;
    let height = sprite.size.height as u16;
    let layers = &sprite.layers;
    let num_frames = sprite.duration;

    // 3. Set up file writer
    let file = File::create(&cli.aseprite_path)?;
    let mut writer = BufWriter::new(file);

    // 4. Write Aseprite Header (with dummy file size)
    let mut header = AsepriteHeader::new(width, height, num_frames as u16);
    header.write(&mut writer)?;

    // 5. Write Layer Chunks (once at the beginning)
    for layer in layers {
        let layer_chunk = LayerChunk {
            flags: if layer.is_visible {
                LayerFlags::VISIBLE | LayerFlags::EDITABLE
            } else {
                LayerFlags::EDITABLE
            },
            layer_type: LayerType::Normal,
            child_level: 0,
            default_width: 0,
            default_height: 0,
            blend_mode: map_blend_mode(layer.blend_mode.as_deref().unwrap_or("normal")),
            opacity: (layer.opacity * 255.0) as u8,
            name: layer.name.clone(),
        };
        let chunk = Chunk::new(ChunkType::Layer, layer_chunk);
        chunk.write(&mut writer)?;
    }

    // Create a map of cels for easy lookup
    let cel_map: HashMap<_, _> = sprite
        .cels
        .iter()
        .map(|c| (c.identifier.clone(), c))
        .collect();

    // Determine the image directory
    let image_dir = if cli.pixaki_path.join("images").join("drawings").is_dir() {
        cli.pixaki_path.join("images").join("drawings")
    } else {
        cli.pixaki_path
    };

    // 6. Loop through frames and write FrameHeader and CelChunks
    for frame_index in 0..num_frames {
        let mut cel_chunks_for_frame = Vec::new();

        for (layer_index, layer) in layers.iter().enumerate() {
            for clip in &layer.clips {
                let in_range = match &clip.range {
                    Some(range) => frame_index >= range.start && frame_index < range.end,
                    None => frame_index == 0, // Assume frame 0 if range is null
                };

                if in_range {
                    if let Some(cel_info) = cel_map.get(&clip.item_identifier) {
                        let image_path = image_dir.join(format!("{}.png", cel_info.identifier));

                        if let Ok(img) = image::open(&image_path) {
                            let rgba_img = img.to_rgba8();
                            let (img_width, img_height) = rgba_img.dimensions();

                            let cel_chunk = CelChunk {
                                layer_index: layer_index as u16,
                                x: cel_info.frame[0][0] as i16,
                                y: cel_info.frame[0][1] as i16,
                                opacity: 255,
                                cel_type: CelType::Compressed,
                                z_index: layer_index as i16,
                                width: img_width as u16,
                                height: img_height as u16,
                                data: rgba_img.into_raw(),
                            };
                            cel_chunks_for_frame.push(cel_chunk);
                        } else {
                            eprintln!("Failed to load image: {:?}", image_path);
                        }
                    }
                }
            }
        }

        // Write FrameHeader
        let frame_header = FrameHeader::new(cel_chunks_for_frame.len() as u16, 100);
        frame_header.write(&mut writer)?;

        // Write CelChunks for the current frame
        for cel_chunk in cel_chunks_for_frame {
            let chunk = Chunk::new(ChunkType::Cel, cel_chunk);
            chunk.write(&mut writer)?;
        }
    }
    
    // 7. Update file size in header
    let file_size = writer.stream_position()?;
    header.file_size = file_size as u32;
    writer.seek(SeekFrom::Start(0))?;
    header.write(&mut writer)?;


    println!(
        "Successfully wrote Aseprite file to {:?}",
        cli.aseprite_path
    );

    Ok(())
}




