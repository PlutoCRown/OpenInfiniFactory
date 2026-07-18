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
/// v5：Material/Stamp 种类后跟字符串 id；设置里材料/印花/漆亦为字符串
pub const VERSION: u16 = 5;
pub const VERSION_V4: u16 = 4;
pub const VERSION_V3: u16 = 3;
pub const VERSION_V2: u16 = 2;
pub const VERSION_V1: u16 = 1;

pub const META_FILE: &str = "meta.json";
pub const BLOCKS_FILE: &str = "blocks.bin";
pub const COVER_FILE: &str = "cover.png";
/// 谜题目录下的水平十字天空盒贴图
pub const SKYBOX_FILE: &str = "skybox.png";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveFormatError {
    UnexpectedEof,
    InvalidMagic,
    UnsupportedVersion(u16),
    UnknownBlockKind(u8),
    UnknownStampMaterialId(String),
    UnknownPaintMaterialId(String),
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
            Self::UnknownStampMaterialId(id) => write!(f, "unknown stamp material id '{id}'"),
            Self::UnknownPaintMaterialId(id) => write!(f, "unknown paint material id '{id}'"),
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

    pub fn read_string(&mut self) -> Result<String, SaveFormatError> {
        let len = self.read_u16()? as usize;
        let bytes = self.read_bytes(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| SaveFormatError::InvalidSettings)
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
    write_scene_section(&mut out, &data.scene_blocks);
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
    let string_ids = version >= VERSION;
    let (scene_blocks, system_blocks, factory_blocks, wire_face_panels) = match version {
        VERSION | VERSION_V4 => {
            let scene_blocks = read_scene_section(&mut cursor, true)?;
            let system_blocks = read_section(&mut cursor, version, string_ids)?;
            let factory_blocks = read_section(&mut cursor, version, string_ids)?;
            let wire_face_panels = read_panel_section(&mut cursor)?;
            (
                scene_blocks,
                system_blocks,
                factory_blocks,
                wire_face_panels,
            )
        }
        VERSION_V3 => {
            // v3：场景段仅字符串 id，无 facing
            let scene_blocks = read_scene_section(&mut cursor, false)?;
            let system_blocks = read_section(&mut cursor, version, false)?;
            let factory_blocks = read_section(&mut cursor, version, false)?;
            let wire_face_panels = read_panel_section(&mut cursor)?;
            (
                scene_blocks,
                system_blocks,
                factory_blocks,
                wire_face_panels,
            )
        }
        VERSION_V2 => {
            // 漏刷兜底：v2 场景段仍为 u8 kind
            let scene_blocks = read_section(&mut cursor, version, false)?;
            let system_blocks = read_section(&mut cursor, version, false)?;
            let factory_blocks = read_section(&mut cursor, version, false)?;
            let wire_face_panels = read_panel_section(&mut cursor)?;
            (
                scene_blocks,
                system_blocks,
                factory_blocks,
                wire_face_panels,
            )
        }
        VERSION_V1 => {
            // v1：无 wire_face_panels 段；场景仍为 u8 kind
            let scene_blocks = read_section(&mut cursor, version, false)?;
            let system_blocks = read_section(&mut cursor, version, false)?;
            let factory_blocks = read_section(&mut cursor, version, false)?;
            (scene_blocks, system_blocks, factory_blocks, Vec::new())
        }
        other => return Err(SaveFormatError::UnsupportedVersion(other)),
    };
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

fn write_scene_section(out: &mut Vec<u8>, blocks: &[SavedBlock]) {
    out.extend_from_slice(&(blocks.len() as u32).to_le_bytes());
    for block in blocks {
        write_scene_block(out, block);
    }
}

fn write_scene_block(out: &mut Vec<u8>, block: &SavedBlock) {
    out.extend_from_slice(&block.x.to_le_bytes());
    out.extend_from_slice(&block.y.to_le_bytes());
    out.extend_from_slice(&block.z.to_le_bytes());
    let string_id = scene_string_id(block.kind);
    let bytes = string_id.as_bytes();
    out.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
    out.extend_from_slice(bytes);
    let has_facing = block.facing.is_some();
    out.push(has_facing as u8);
    if let Some(facing) = block.facing {
        out.push(encode_facing(facing));
    }
}

fn scene_string_id(kind: BlockKind) -> String {
    match kind {
        BlockKind::Scene(id) => oif_sim::blocks::scene_catalog()
            .string_id(id)
            .unwrap_or(oif_sim::blocks::FALLBACK_SCENE_STRING_ID)
            .to_string(),
        // 防御：不应出现
        other => format!("{other:?}").to_ascii_lowercase(),
    }
}

fn read_scene_section(
    cursor: &mut Cursor<'_>,
    with_facing: bool,
) -> Result<Vec<SavedBlock>, SaveFormatError> {
    let count = cursor.read_u32()? as usize;
    let mut blocks = Vec::with_capacity(count);
    for _ in 0..count {
        blocks.push(read_scene_block(cursor, with_facing)?);
    }
    Ok(blocks)
}

fn read_scene_block(
    cursor: &mut Cursor<'_>,
    with_facing: bool,
) -> Result<SavedBlock, SaveFormatError> {
    let x = cursor.read_i32()?;
    let y = cursor.read_i32()?;
    let z = cursor.read_i32()?;
    let string_id = cursor.read_string()?;
    let kind = BlockKind::Scene(oif_sim::blocks::resolve_scene_id(&string_id));
    let facing = if with_facing {
        let flags = cursor.read_u8()?;
        if flags & 1 != 0 {
            Some(decode_facing(cursor.read_u8()?)?)
        } else {
            None
        }
    } else {
        None
    };
    Ok(SavedBlock {
        x,
        y,
        z,
        kind,
        facing,
        settings: None,
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

fn write_string(out: &mut Vec<u8>, value: &str) {
    let bytes = value.as_bytes();
    let len = bytes.len().min(u16::MAX as usize) as u16;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(&bytes[..len as usize]);
}

fn write_block(out: &mut Vec<u8>, block: &SavedBlock) {
    out.extend_from_slice(&block.x.to_le_bytes());
    out.extend_from_slice(&block.y.to_le_bytes());
    out.extend_from_slice(&block.z.to_le_bytes());
    out.push(encode_kind(block.kind));
    // v5：Material/Stamp 在种类字节后写字符串 id
    match block.kind {
        BlockKind::Material(id) => {
            oif_sim::blocks::ensure_fallback_material_catalog();
            let catalog = oif_sim::blocks::material_catalog();
            let string_id = catalog
                .string_id(id)
                .unwrap_or(oif_sim::blocks::FALLBACK_MATERIAL_STRING_ID);
            write_string(out, string_id);
        }
        BlockKind::Stamp(id) => {
            oif_sim::blocks::ensure_fallback_stamp_catalog();
            let catalog = oif_sim::blocks::stamp_catalog();
            let string_id = catalog.string_id(id).unwrap_or("unknown");
            write_string(out, string_id);
        }
        _ => {}
    }
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

fn read_section(
    cursor: &mut Cursor<'_>,
    version: u16,
    string_ids: bool,
) -> Result<Vec<SavedBlock>, SaveFormatError> {
    let count = cursor.read_u32()? as usize;
    let mut blocks = Vec::with_capacity(count);
    for _ in 0..count {
        blocks.push(read_block(cursor, version, string_ids)?);
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

fn read_block(
    cursor: &mut Cursor<'_>,
    version: u16,
    string_ids: bool,
) -> Result<SavedBlock, SaveFormatError> {
    let x = cursor.read_i32()?;
    let y = cursor.read_i32()?;
    let z = cursor.read_i32()?;
    let kind_u8 = cursor.read_u8()?;
    let kind = if version >= VERSION {
        match kind_u8 {
            29 => {
                let string_id = cursor.read_string()?;
                BlockKind::Material(oif_sim::blocks::resolve_material_id(&string_id))
            }
            38 => {
                let string_id = cursor.read_string()?;
                oif_sim::blocks::ensure_fallback_stamp_catalog();
                oif_sim::blocks::stamp_catalog()
                    .id_by_string(&string_id)
                    .map(BlockKind::Stamp)
                    .ok_or(SaveFormatError::UnknownStampMaterialId(string_id))?
            }
            other => decode_kind(other)?,
        }
    } else {
        decode_kind(kind_u8)?
    };
    let flags = cursor.read_u8()?;
    let facing = if flags & 1 != 0 {
        Some(decode_facing(cursor.read_u8()?)?)
    } else {
        None
    };
    let settings = if flags & 2 != 0 {
        Some(read_settings(cursor, kind, string_ids)?)
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
