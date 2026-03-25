use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{Result, Write};

// Basic data types
pub type BYTE = u8;
pub type WORD = u16;
pub type SHORT = i16;
pub type DWORD = u32;
pub type LONG = i32;
pub type FIXED = f32; // 16.16 fixed point
pub type QWORD = u64;
pub type LONG64 = i64;

// Function to write a BYTE (u8)
pub fn write_byte<W: Write>(writer: &mut W, value: BYTE) -> Result<()> {
    writer.write_all(&[value])
}

// Function to write a WORD (u16) in little-endian
pub fn write_word<W: Write>(writer: &mut W, value: WORD) -> Result<()> {
    writer.write_all(&value.to_le_bytes())
}

// Function to write a SHORT (i16) in little-endian
pub fn write_short<W: Write>(writer: &mut W, value: SHORT) -> Result<()> {
    writer.write_all(&value.to_le_bytes())
}

// Function to write a DWORD (u32) in little-endian
pub fn write_dword<W: Write>(writer: &mut W, value: DWORD) -> Result<()> {
    writer.write_all(&value.to_le_bytes())
}

// Function to write a LONG (i32) in little-endian
pub fn write_long<W: Write>(writer: &mut W, value: LONG) -> Result<()> {
    writer.write_all(&value.to_le_bytes())
}

// Function to write a string
pub fn write_string<W: Write>(writer: &mut W, value: &str) -> Result<()> {
    write_word(writer, value.len() as WORD)?;
    writer.write_all(value.as_bytes())
}

// Aseprite Header
#[derive(Debug)]
pub struct AsepriteHeader {
    pub file_size: DWORD,
    pub magic_number: WORD,
    pub frames: WORD,
    pub width: WORD,
    pub height: WORD,
    pub color_depth: WORD,
    pub flags: DWORD,
    pub speed: WORD, // ms between frames
    pub zero1: DWORD,
    pub zero2: DWORD,
    pub transparent_index: BYTE,
    pub ignore1: [BYTE; 3],
    pub num_colors: WORD,
    pub pixel_width: BYTE,
    pub pixel_height: BYTE,
    pub grid_x: SHORT,
    pub grid_y: SHORT,
    pub grid_width: WORD,
    pub grid_height: WORD,
    pub ignore2: [BYTE; 84],
}

impl AsepriteHeader {
    pub fn new(width: WORD, height: WORD, frames: WORD) -> Self {
        Self {
            file_size: 0, // To be calculated later
            magic_number: 0xA5E0,
            frames,
            width,
            height,
            color_depth: 32, // RGBA
            flags: 1, // Layer opacity is valid
            speed: 100,
            zero1: 0,
            zero2: 0,
            transparent_index: 0,
            ignore1: [0; 3],
            num_colors: 0,
            pixel_width: 0,
            pixel_height: 0,
            grid_x: 0,
            grid_y: 0,
            grid_width: 0,
            grid_height: 0,
            ignore2: [0; 84],
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        write_dword(writer, self.file_size)?;
        write_word(writer, self.magic_number)?;
        write_word(writer, self.frames)?;
        write_word(writer, self.width)?;
        write_word(writer, self.height)?;
        write_word(writer, self.color_depth)?;
        write_dword(writer, self.flags)?;
        write_word(writer, self.speed)?;
        write_dword(writer, self.zero1)?;
        write_dword(writer, self.zero2)?;
        write_byte(writer, self.transparent_index)?;
        writer.write_all(&self.ignore1)?;
        write_word(writer, self.num_colors)?;
        write_byte(writer, self.pixel_width)?;
        write_byte(writer, self.pixel_height)?;
        write_short(writer, self.grid_x)?;
        write_short(writer, self.grid_y)?;
        write_word(writer, self.grid_width)?;
        write_word(writer, self.grid_height)?;
        writer.write_all(&self.ignore2)?;
        Ok(())
    }
}

// Frame Header
#[derive(Debug)]
pub struct FrameHeader {
    pub size: DWORD,
    pub magic_number: WORD,
    pub chunks: WORD,
    pub duration: WORD,
}

impl FrameHeader {
    pub fn new(chunks: WORD, duration: WORD) -> Self {
        Self {
            size: 0, // To be calculated later
            magic_number: 0xF1FA,
            chunks,
            duration,
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        write_dword(writer, self.size)?;
        write_word(writer, self.magic_number)?;
        write_word(writer, self.chunks)?;
        write_word(writer, self.duration)?;
        // Skip 2 bytes
        writer.write_all(&[0; 2])?;
        // Skip 4 bytes
        writer.write_all(&[0; 4])?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum ChunkType {
    OldPalette = 0x0004,
    OldPalette2 = 0x0011,
    Layer = 0x2004,
    Cel = 0x2005,
    CelExtra = 0x2006,
    ColorProfile = 0x2007,
    ExternalFiles = 0x2008,
    Mask = 0x2016,
    Path = 0x2017,
    Tags = 0x2018,
    Palette = 0x2019,
    UserData = 0x2020,
    Slice = 0x2022,
    Tileset = 0x2023,
}

pub struct Chunk<T> {
    pub size: DWORD,
    pub chunk_type: ChunkType,
    pub data: T,
}

pub trait ChunkData {
    fn size(&self) -> DWORD;
    fn write<W: Write>(&self, writer: &mut W) -> Result<()>;
}

impl<T: ChunkData> Chunk<T> {
    pub fn new(chunk_type: ChunkType, data: T) -> Self {
        Self {
            size: 0, // To be calculated later
            chunk_type,
            data,
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let size = self.data.size() + 6; // 4 bytes for size, 2 for type
        write_dword(writer, size)?;
        write_word(writer, self.chunk_type as WORD)?;
        self.data.write(writer)?;
        Ok(())
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct LayerFlags: WORD {
        const VISIBLE = 1;
        const EDITABLE = 2;
        const LOCK_MOVEMENT = 4;
        const BACKGROUND = 8;
        const PREFER_LINKED_CELS = 16;
        const COLLAPSED = 32;
        const REFERENCE = 64;
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum LayerType {
    Normal = 0,
    Group = 1,
    Tilemap = 2,
}

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum BlendMode {
    Normal = 0,
    Multiply = 1,
    Screen = 2,
    Overlay = 3,
    Darken = 4,
    Lighten = 5,
    ColorDodge = 6,
    ColorBurn = 7,
    HardLight = 8,
    SoftLight = 9,
    Difference = 10,
    Exclusion = 11,
    Hue = 12,
    Saturation = 13,
    Color = 14,
    Luminosity = 15,
}


pub struct LayerChunk {
    pub flags: LayerFlags,
    pub layer_type: LayerType,
    pub child_level: WORD,
    pub default_width: WORD,
    pub default_height: WORD,
    pub blend_mode: BlendMode,
    pub opacity: BYTE,
    pub name: String,
}

impl ChunkData for LayerChunk {
    fn size(&self) -> DWORD {
        2 + // flags
        2 + // type
        2 + // child level
        2 + // width
        2 + // height
        2 + // blend mode
        1 + // opacity
        3 + // reserved
        2 + self.name.len() as DWORD // name
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        write_word(writer, self.flags.bits())?;
        write_word(writer, self.layer_type as WORD)?;
        write_word(writer, self.child_level)?;
        write_word(writer, self.default_width)?;
        write_word(writer, self.default_height)?;
        write_word(writer, self.blend_mode as WORD)?;
        write_byte(writer, self.opacity)?;
        writer.write_all(&[0; 3])?; // for future use
        write_string(writer, &self.name)?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum CelType {
    Raw = 0,
    Linked = 1,
    Compressed = 2,
}

pub struct CelChunk {
    pub layer_index: WORD,
    pub x: SHORT,
    pub y: SHORT,
    pub opacity: BYTE,
    pub cel_type: CelType,
    pub z_index: SHORT,
    pub width: WORD,
    pub height: WORD,
    pub data: Vec<u8>,
}

impl ChunkData for CelChunk {
    fn size(&self) -> DWORD {
        let base_size = 2 + // layer index
            2 + // x
            2 + // y
            1 + // opacity
            2 + // cel type
            7; // for future use

        let data_size = match self.cel_type {
            CelType::Raw => 2 + 2 + self.data.len() as DWORD,
            CelType::Compressed => {
                let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
                e.write_all(&self.data).unwrap();
                let compressed_data = e.finish().unwrap();
                2 + 2 + compressed_data.len() as DWORD
            }
            _ => 0,
        };
        base_size + data_size
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        write_word(writer, self.layer_index)?;
        write_short(writer, self.x)?;
        write_short(writer, self.y)?;
        write_byte(writer, self.opacity)?;
        write_word(writer, self.cel_type as WORD)?;
        writer.write_all(&[0; 7])?; // for future use
        
        match self.cel_type {
            CelType::Raw => {
                write_word(writer, self.width)?;
                write_word(writer, self.height)?;
                writer.write_all(&self.data)?;
            }
            CelType::Compressed => {
                write_word(writer, self.width)?;
                write_word(writer, self.height)?;
                let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
                e.write_all(&self.data)?;
                let compressed_data = e.finish()?;
                writer.write_all(&compressed_data)?;
            }
            // TODO: Implement other cel types
            _ => unimplemented!(),
        }

        Ok(())
    }
}
