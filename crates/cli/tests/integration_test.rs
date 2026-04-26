use std::process::Command;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_fox_smile() {
    let pixaki_path = PathBuf::from("tests/data/fox_smile.pixaki");
    let output_path = PathBuf::from("tests/data/fox_smile.aseprite");
    
    // Ensure output doesn't exist
    if output_path.exists() {
        fs::remove_file(&output_path).unwrap();
    }

    let status = Command::new("cargo")
        .args(["run", "--", pixaki_path.to_str().unwrap(), output_path.to_str().unwrap()])
        .status()
        .expect("Failed to execute command");

    assert!(status.success());
    assert!(output_path.exists());
    
    // Optional: Clean up
    fs::remove_file(&output_path).unwrap();
}

#[test]
fn test_fox_walk_2010s() {
    let pixaki_path = PathBuf::from("tests/data/fox_walk.pixaki");
    let output_path = PathBuf::from("tests/data/fox_walk.aseprite");
    
    // Ensure output doesn't exist
    if output_path.exists() {
        fs::remove_file(&output_path).unwrap();
    }

    let status = Command::new("cargo")
        .args(["run", "--", pixaki_path.to_str().unwrap(), output_path.to_str().unwrap()])
        .status()
        .expect("Failed to execute command");

    // This will likely fail for now as it's an old format
    assert!(status.success());
    assert!(output_path.exists());
    
    // Optional: Clean up
    fs::remove_file(&output_path).unwrap();
}

#[cfg(feature = "image")]
#[test]
fn test_image_export_fox_smile() {
    let pixaki_path = PathBuf::from("tests/data/fox_smile.pixaki");
    let document_path = pixaki_path.join("document.json");
    let json_str = fs::read_to_string(document_path).expect("Unable to read document.json");
    let doc_v3: pixaki_v3::Document = serde_json::from_str(&json_str).expect("Unable to parse document.json");
    let doc = pixaki_v3_converter::convert(doc_v3, &pixaki_path).expect("Failed to convert pixaki_v3::Document");

    assert!(!doc.cels.is_empty(), "Document should have at least one cel");
    let first_cel_image = doc.cels[0].image.clone();

    let rgba_image: image::RgbaImage = first_cel_image.into();
    assert_eq!(rgba_image.width() as u16, doc.width);
    assert_eq!(rgba_image.height() as u16, doc.height);
}

#[cfg(feature = "image")]
#[test]
fn test_image_export_fox_walk() {
    let pixaki_path = PathBuf::from("tests/data/fox_walk.pixaki");
    let plist_path = pixaki_path.join("DocumentInfo.plist");
    let doc_v2: pixaki_v2::Document = plist::from_file(plist_path).expect("Failed to parse DocumentInfo.plist");
    let doc = pixaki_v2_converter::convert(doc_v2, &pixaki_path).expect("Failed to convert pixaki_v2::Document");

    assert!(!doc.cels.is_empty(), "Document should have at least one cel");
    let first_cel_image = doc.cels[0].image.clone();

    let rgba_image: image::RgbaImage = first_cel_image.into();
    assert_eq!(rgba_image.width() as u16, doc.width);
    assert_eq!(rgba_image.height() as u16, doc.height);
}

#[cfg(feature = "image")]
#[test]
fn test_image_export_frame_psp() {
    let psp_path = PathBuf::from("tests/data/frame.psp");
    let json_str = fs::read_to_string(&psp_path).expect("Unable to read frame.psp");
    let doc_psp: pixel_studio_pro_v2::Document = serde_json::from_str(&json_str).expect("Unable to parse frame.psp");
    let doc = pixel_studio_pro_v2_converter::convert(doc_psp).expect("Failed to convert pixel_studio_pro_v2::Document");

    assert!(!doc.cels.is_empty(), "Document should have at least one cel");
    let first_cel_image = doc.cels[0].image.clone();

    let rgba_image: image::RgbaImage = first_cel_image.clone().into();
    // width and height match the cel image's width and height
    assert_eq!(rgba_image.width() as u16, first_cel_image.width);
    assert_eq!(rgba_image.height() as u16, first_cel_image.height);
}
