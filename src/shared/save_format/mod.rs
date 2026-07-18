mod block_kind;
mod settings;

use bevy::prelude::*;

use crate::game::blocks::save_stores_facing;
use crate::game::blocks::{BlockData, BlockKind};
use crate::game::world::direction::Facing;
use crate::game::world::grid::BlockSettings;

pub use block_kind::{decode_kind, encode_kind};
pub use settings::{read_settings, write_settings};

pub const MAGIC: &[u8; 4] = b"OIF\0";
pub const VERSION: u16 = 2;

pub const META_FILE: &str = "meta.json";
pub const BLOCKS_FILE: &str = "blocks.bin";
pub const COVER_FILE: &str = "cover.png";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveFormatError {
    UnexpectedEof,
    InvalidMagic,
    UnsupportedVersion(u16),
    UnknownBlockKind(u8),
    InvalidSettings,
    InvalidFacing(u8),
}

impl std::fmt::Display for SaveFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "unexpected end of save data"),
            Self::InvalidMagic => write!(f, "invalid blocks.bin magic"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported blocks.bin version {version}")
            }
            Self::UnknownBlockKind(id) => write!(f, "unknown block kind id {id}"),
            Self::InvalidSettings => write!(f, "invalid block settings payload"),
            Self::InvalidFacing(value) => write!(f, "invalid facing value {value}"),
        }
    }
}

impl std::error::Error for SaveFormatError {}

/// blocks.bin 读取游标
pub struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn remaining(&self) -> &[u8] {
        &self.data[self.pos..]
    }

    pub fn read_u8(&mut self) -> Result<u8, SaveFormatError> {
        if self.pos >= self.data.len() {
            return Err(SaveFormatError::UnexpectedEof);
        }
        let value = self.data[self.pos];
        self.pos += 1;
        Ok(value)
    }

    pub fn read_u16(&mut self) -> Result<u16, SaveFormatError> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub fn read_u32(&mut self) -> Result<u32, SaveFormatError> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn read_u64(&mut self) -> Result<u64, SaveFormatError> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    pub fn read_i32(&mut self) -> Result<i32, SaveFormatError> {
        let bytes = self.read_bytes(4)?;
        Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], SaveFormatError> {
        if self.pos + len > self.data.len() {
            return Err(SaveFormatError::UnexpectedEof);
        }
        let slice = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(slice)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SavedBlock {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub kind: BlockKind,
    pub facing: Option<Facing>,
    pub settings: Option<BlockSettings>,
}

impl SavedBlock {
    pub fn pos(&self) -> IVec3 {
        IVec3::new(self.x, self.y, self.z)
    }

    pub fn to_block_data(&self) -> BlockData {
        BlockData::new(self.kind, self.facing.unwrap_or(Facing::North))
    }

    pub fn from_block_data(pos: IVec3, data: BlockData) -> Self {
        let facing = save_stores_facing(data.kind).then_some(data.facing);
        Self {
            x: pos.x,
            y: pos.y,
            z: pos.z,
            kind: data.kind,
            facing,
            settings: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SavedWireFacePanel {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub nx: i32,
    pub ny: i32,
    pub nz: i32,
}

impl SavedWireFacePanel {
    pub fn pos(&self) -> IVec3 {
        IVec3::new(self.x, self.y, self.z)
    }

    pub fn normal(&self) -> IVec3 {
        IVec3::new(self.nx, self.ny, self.nz)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SaveBlocksData {
    pub scene_blocks: Vec<SavedBlock>,
    pub system_blocks: Vec<SavedBlock>,
    pub factory_blocks: Vec<SavedBlock>,
    pub wire_face_panels: Vec<SavedWireFacePanel>,
}

pub fn encode_blocks(data: &SaveBlocksData) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    write_section(&mut out, &data.scene_blocks);
    write_section(&mut out, &data.system_blocks);
    write_section(&mut out, &data.factory_blocks);
    write_panel_section(&mut out, &data.wire_face_panels);
    out
}

pub fn decode_blocks(bytes: &[u8]) -> Result<SaveBlocksData, SaveFormatError> {
    let mut cursor = Cursor::new(bytes);
    if cursor.read_bytes(4)? != MAGIC {
        return Err(SaveFormatError::InvalidMagic);
    }
    let version = cursor.read_u16()?;
    if version != VERSION {
        return Err(SaveFormatError::UnsupportedVersion(version));
    }
    let scene_blocks = read_section(&mut cursor)?;
    let system_blocks = read_section(&mut cursor)?;
    let factory_blocks = read_section(&mut cursor)?;
    let wire_face_panels = read_panel_section(&mut cursor)?;
    if !cursor.remaining().is_empty() {
        return Err(SaveFormatError::UnexpectedEof);
    }
    Ok(SaveBlocksData {
        scene_blocks,
        system_blocks,
        factory_blocks,
        wire_face_panels,
    })
}

fn write_section(out: &mut Vec<u8>, blocks: &[SavedBlock]) {
    out.extend_from_slice(&(blocks.len() as u32).to_le_bytes());
    for block in blocks {
        write_block(out, block);
    }
}

fn write_panel_section(out: &mut Vec<u8>, panels: &[SavedWireFacePanel]) {
    out.extend_from_slice(&(panels.len() as u32).to_le_bytes());
    for panel in panels {
        out.extend_from_slice(&panel.x.to_le_bytes());
        out.extend_from_slice(&panel.y.to_le_bytes());
        out.extend_from_slice(&panel.z.to_le_bytes());
        out.extend_from_slice(&panel.nx.to_le_bytes());
        out.extend_from_slice(&panel.ny.to_le_bytes());
        out.extend_from_slice(&panel.nz.to_le_bytes());
    }
}

fn write_block(out: &mut Vec<u8>, block: &SavedBlock) {
    out.extend_from_slice(&block.x.to_le_bytes());
    out.extend_from_slice(&block.y.to_le_bytes());
    out.extend_from_slice(&block.z.to_le_bytes());
    out.push(encode_kind(block.kind));
    let has_facing = block.facing.is_some();
    let has_settings = block.settings.is_some();
    out.push((has_facing as u8) | ((has_settings as u8) << 1));
    if let Some(facing) = block.facing {
        out.push(encode_facing(facing));
    }
    if let Some(settings) = &block.settings {
        write_settings(out, block.kind, settings);
    }
}

fn read_section(cursor: &mut Cursor<'_>) -> Result<Vec<SavedBlock>, SaveFormatError> {
    let count = cursor.read_u32()? as usize;
    let mut blocks = Vec::with_capacity(count);
    for _ in 0..count {
        blocks.push(read_block(cursor)?);
    }
    Ok(blocks)
}

fn read_panel_section(cursor: &mut Cursor<'_>) -> Result<Vec<SavedWireFacePanel>, SaveFormatError> {
    let count = cursor.read_u32()? as usize;
    let mut panels = Vec::with_capacity(count);
    for _ in 0..count {
        panels.push(SavedWireFacePanel {
            x: cursor.read_i32()?,
            y: cursor.read_i32()?,
            z: cursor.read_i32()?,
            nx: cursor.read_i32()?,
            ny: cursor.read_i32()?,
            nz: cursor.read_i32()?,
        });
    }
    Ok(panels)
}

fn read_block(cursor: &mut Cursor<'_>) -> Result<SavedBlock, SaveFormatError> {
    let x = cursor.read_i32()?;
    let y = cursor.read_i32()?;
    let z = cursor.read_i32()?;
    let kind = decode_kind(cursor.read_u8()?)?;
    let flags = cursor.read_u8()?;
    let facing = if flags & 1 != 0 {
        Some(decode_facing(cursor.read_u8()?)?)
    } else {
        None
    };
    let settings = if flags & 2 != 0 {
        Some(read_settings(cursor, kind)?)
    } else {
        None
    };
    Ok(SavedBlock {
        x,
        y,
        z,
        kind,
        facing,
        settings,
    })
}

fn encode_facing(facing: Facing) -> u8 {
    match facing {
        Facing::North => 0,
        Facing::East => 1,
        Facing::South => 2,
        Facing::West => 3,
    }
}

fn decode_facing(value: u8) -> Result<Facing, SaveFormatError> {
    Ok(match value {
        0 => Facing::North,
        1 => Facing::East,
        2 => Facing::South,
        3 => Facing::West,
        _ => return Err(SaveFormatError::InvalidFacing(value)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::blocks::{BlockData, Facing, MaterialKind, StampColor};
    use crate::game::world::grid::{
        ConverterMode, ConverterSettings, GeneratorMode, GeneratorSettings, StamperSettings,
        TeleportSettings,
    };

    #[test]
    fn blocks_round_trip_preserves_settings_and_facing() {
        let data = SaveBlocksData {
            scene_blocks: vec![SavedBlock {
                x: 1,
                y: 2,
                z: 3,
                kind: BlockKind::Platform,
                facing: None,
                settings: None,
            }],
            system_blocks: vec![
                SavedBlock {
                    x: 4,
                    y: 1,
                    z: 0,
                    kind: BlockKind::Generator,
                    facing: Some(Facing::East),
                    settings: Some(BlockSettings::Generator(GeneratorSettings {
                        mode: GeneratorMode::Period {
                            period: 9,
                            offset: 2,
                        },
                        material: MaterialKind::Copper,
                    })),
                },
                SavedBlock {
                    x: 5,
                    y: 1,
                    z: 0,
                    kind: BlockKind::TeleportEntrance,
                    facing: Some(Facing::North),
                    settings: Some(BlockSettings::Teleport(TeleportSettings {
                        name: "In".to_string(),
                        pair: Some(IVec3::new(6, 1, 0)),
                    })),
                },
            ],
            factory_blocks: vec![SavedBlock::from_block_data(
                IVec3::new(0, 0, 1),
                BlockData::new(BlockKind::Conveyor, Facing::South),
            )],
            wire_face_panels: vec![SavedWireFacePanel {
                x: 0,
                y: 0,
                z: 1,
                nx: 0,
                ny: 1,
                nz: 0,
            }],
        };

        let encoded = encode_blocks(&data);
        let decoded = decode_blocks(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn decode_rejects_unknown_kind() {
        let data = SaveBlocksData {
            factory_blocks: vec![SavedBlock::from_block_data(
                IVec3::ZERO,
                BlockData::new(BlockKind::Platform, Facing::North),
            )],
            ..Default::default()
        };
        let mut bytes = encode_blocks(&data);
        bytes[30] = 255;
        assert!(matches!(
            decode_blocks(&bytes),
            Err(SaveFormatError::UnknownBlockKind(255))
        ));
    }

    #[test]
    fn stamper_settings_round_trip() {
        let data = SaveBlocksData {
            system_blocks: vec![SavedBlock {
                x: 0,
                y: 0,
                z: 0,
                kind: BlockKind::Stamper,
                facing: Some(Facing::West),
                settings: Some(BlockSettings::Stamper(StamperSettings {
                    color: StampColor::Blue,
                })),
            }],
            ..Default::default()
        };
        assert_eq!(decode_blocks(&encode_blocks(&data)).unwrap(), data);
    }

    #[test]
    fn converter_settings_round_trip() {
        let data = SaveBlocksData {
            system_blocks: vec![SavedBlock {
                x: 0,
                y: 0,
                z: 0,
                kind: BlockKind::Converter,
                facing: Some(Facing::North),
                settings: Some(BlockSettings::Converter(ConverterSettings {
                    mode: ConverterMode::SpecificInput,
                    input: MaterialKind::Iron,
                    output: MaterialKind::Copper,
                })),
            }],
            ..Default::default()
        };
        assert_eq!(decode_blocks(&encode_blocks(&data)).unwrap(), data);
    }
}
