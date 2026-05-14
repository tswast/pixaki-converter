use anyhow::{Context, Result, anyhow};
use clap::Parser;
use std::fs;
use std::path::Path;

/// A simple program to convert Pixaki, Pixel Studio Pro, PSD, and Aseprite files.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to the input file or directory (.pixaki, .psp, .psd, .ase, .aseprite)
    input_path: std::path::PathBuf,

    /// The path to the output file (.ase, .aseprite, or .png)
    output_path: std::path::PathBuf,

    /// Extract a timelapse animation if the input format supports it (e.g. .psp)
    #[arg(long)]
    timelapse: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let doc = if cli.input_path.is_file()
        && cli
            .input_path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("psp"))
    {
        handle_psp_format(&cli.input_path, cli.timelapse)?
    } else if cli.input_path.is_file()
        && cli
            .input_path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("psd"))
    {
        handle_psd_format(&cli.input_path)?
    } else if cli.input_path.is_file()
        && cli
            .input_path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| {
                ext.eq_ignore_ascii_case("ase") || ext.eq_ignore_ascii_case("aseprite")
            })
    {
        handle_aseprite_format(&cli.input_path)?
    } else if cli.input_path.join("document.json").exists() {
        handle_modern_format(&cli.input_path)?
    } else if cli.input_path.join("DocumentInfo.plist").exists() {
        handle_legacy_format(&cli.input_path)?
    } else {
        return Err(anyhow!(
            "No valid .psp, .psd, .ase, .aseprite file, or document.json/DocumentInfo.plist found in the given path"
        ));
    };

    let ext = cli
        .output_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "ase" || ext == "aseprite" {
        let aseprite_file = aseprite_converter::convert(doc)?;

        let mut buffer = Vec::new();
        aseprite_file
            .write_to(&mut buffer)
            .map_err(|e| anyhow!("Failed to write Aseprite file: {}", e))?;
        fs::write(&cli.output_path, buffer)?;

        println!("Successfully wrote Aseprite file to {:?}", cli.output_path);
    } else if ext == "png" {
        #[cfg(feature = "tiny-skia")]
        {
            let img = doc.render_skia();
            img.save_png(&cli.output_path)
                .context("Failed to write PNG file")?;
            println!("Successfully wrote PNG file to {:?}", cli.output_path);
        }
        #[cfg(all(feature = "image", not(feature = "tiny-skia")))]
        {
            let img = doc.render();
            img.save(&cli.output_path)
                .context("Failed to write PNG file")?;
            println!("Successfully wrote PNG file to {:?}", cli.output_path);
        }
        #[cfg(not(any(feature = "image", feature = "tiny-skia")))]
        {
            return Err(anyhow!(
                "PNG export requires the 'image' or 'tiny-skia' feature to be enabled"
            ));
        }
    } else {
        return Err(anyhow!(
            "Unsupported output format: '{}'. Supported formats are .ase, .aseprite, and .png",
            ext
        ));
    }

    Ok(())
}

fn handle_modern_format(pixaki_path: &Path) -> Result<pixel_art::Document> {
    let document_path = pixaki_path.join("document.json");
    let json_str = fs::read_to_string(document_path)?;
    let doc_v3: pixaki_v3::Document =
        serde_json::from_str(&json_str).context("Unable to parse document.json")?;

    pixaki_v3_converter::convert(doc_v3, pixaki_path)
}

fn handle_legacy_format(pixaki_path: &Path) -> Result<pixel_art::Document> {
    let plist_path = pixaki_path.join("DocumentInfo.plist");
    let doc_v2: pixaki_v2::Document =
        plist::from_file(plist_path).context("Failed to parse DocumentInfo.plist")?;

    pixaki_v2_converter::convert(doc_v2, pixaki_path)
}

fn handle_psp_format(psp_path: &Path, timelapse: bool) -> Result<pixel_art::Document> {
    let json_str = fs::read_to_string(psp_path)?;
    let doc_psp: pixel_studio_pro_v2::Document =
        serde_json::from_str(&json_str).context("Unable to parse .psp JSON document")?;

    pixel_studio_pro_v2_converter::convert(doc_psp, timelapse)
}

fn handle_psd_format(psd_path: &Path) -> Result<pixel_art::Document> {
    let bytes = fs::read(psd_path)?;
    psd_converter::convert(&bytes).context("Failed to parse .psd file")
}

fn handle_aseprite_format(ase_path: &Path) -> Result<pixel_art::Document> {
    let file = fs::File::open(ase_path)?;
    let aseprite_file =
        aseprite::AsepriteFile::from_reader(file).context("Failed to parse .aseprite file")?;
    aseprite_converter::reader::parse(aseprite_file)
}
