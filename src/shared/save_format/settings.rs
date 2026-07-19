use bevy::prelude::*;

use crate::game::blocks::{
    BlockKind, MaterialBlockId, PaintMaterialId, StampMaterialId, ensure_fallback_material_catalog,
    ensure_fallback_paint_catalog, ensure_fallback_stamp_catalog, fallback_material_id,
    material_catalog, paint_catalog, paint_id_by_string, resolve_material_id, stamp_catalog,
    stamp_id_by_string,
};
use crate::game::world::grid::{
    BlockSettings, ConverterMode, ConverterSettings, GeneratorMode, GeneratorSettings,
    GoalSettings, RollerSettings, SignDisplay, SignSettings, StamperSettings, TeleportSettings,
};

use super::Cursor;
use super::SaveFormatError;

/// 写入方块设置（v5：材料/印花/漆均为字符串 id）
pub fn write_settings(out: &mut Vec<u8>, kind: BlockKind, settings: &BlockSettings) {
    match (kind, settings) {
        (BlockKind::Generator, BlockSettings::Generator(value)) => write_generator(out, *value),
        (BlockKind::Goal, BlockSettings::Goal(value)) => write_goal(out, *value),
        (BlockKind::Stamper, BlockSettings::Stamper(value)) => write_stamper(out, *value),
        (BlockKind::Roller, BlockSettings::Roller(value)) => write_roller(out, *value),
        (BlockKind::Converter, BlockSettings::Converter(value)) => write_converter(out, *value),
        (BlockKind::TeleportEntrance | BlockKind::TeleportExit, BlockSettings::Teleport(value)) => {
            write_teleport(out, value)
        }
        (BlockKind::Sign, BlockSettings::Sign(value)) => write_sign(out, value),
        _ => {}
    }
}

/// 读取方块设置；`string_ids` 为真时用字符串，否则兼容旧 u8 表
pub fn read_settings(
    cursor: &mut Cursor<'_>,
    kind: BlockKind,
    string_ids: bool,
) -> Result<BlockSettings, SaveFormatError> {
    Ok(match kind {
        BlockKind::Generator => BlockSettings::Generator(read_generator(cursor, string_ids)?),
        BlockKind::Goal => BlockSettings::Goal(read_goal(cursor, string_ids)?),
        BlockKind::Stamper => BlockSettings::Stamper(read_stamper(cursor, string_ids)?),
        BlockKind::Roller => BlockSettings::Roller(read_roller(cursor, string_ids)?),
        BlockKind::Converter => BlockSettings::Converter(read_converter(cursor, string_ids)?),
        BlockKind::TeleportEntrance | BlockKind::TeleportExit => {
            BlockSettings::Teleport(read_teleport(cursor)?)
        }
        BlockKind::Sign => BlockSettings::Sign(read_sign(cursor, string_ids)?),
        _ => return Err(SaveFormatError::InvalidSettings),
    })
}

fn write_string(out: &mut Vec<u8>, value: &str) {
    let bytes = value.as_bytes();
    let len = bytes.len().min(u16::MAX as usize) as u16;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(&bytes[..len as usize]);
}

fn write_generator(out: &mut Vec<u8>, settings: GeneratorSettings) {
    match settings.mode {
        GeneratorMode::Period { period, offset } => {
            out.push(0);
            out.extend_from_slice(&period.to_le_bytes());
            out.extend_from_slice(&offset.to_le_bytes());
        }
        GeneratorMode::Link { anchor } => {
            out.push(1);
            if let Some(pos) = anchor {
                out.push(1);
                out.extend_from_slice(&pos.x.to_le_bytes());
                out.extend_from_slice(&pos.y.to_le_bytes());
                out.extend_from_slice(&pos.z.to_le_bytes());
            } else {
                out.push(0);
            }
        }
    }
    write_material_id(out, settings.material);
    out.push(encode_settings_facing(settings.facing));
}

fn read_generator(
    cursor: &mut Cursor<'_>,
    string_ids: bool,
) -> Result<GeneratorSettings, SaveFormatError> {
    let mode_tag = cursor.read_u8()?;
    let mode = match mode_tag {
        0 => GeneratorMode::Period {
            period: cursor.read_u64()?,
            offset: cursor.read_u64()?,
        },
        1 => GeneratorMode::Link {
            anchor: if cursor.read_u8()? == 1 {
                Some(IVec3::new(
                    cursor.read_i32()?,
                    cursor.read_i32()?,
                    cursor.read_i32()?,
                ))
            } else {
                None
            },
        },
        _ => return Err(SaveFormatError::InvalidSettings),
    };
    Ok(GeneratorSettings {
        mode,
        material: read_material_id(cursor, string_ids)?,
        facing: decode_settings_facing(cursor.read_u8()?)?,
    })
}

fn write_goal(out: &mut Vec<u8>, settings: GoalSettings) {
    write_material_id(out, settings.material);
    out.push(encode_settings_facing(settings.facing));
    for stamp in settings.stamps {
        match stamp {
            Some(id) => {
                out.push(1);
                write_stamp_id(out, id);
            }
            None => out.push(0),
        }
    }
    for paint in settings.paints {
        match paint {
            Some(id) => {
                out.push(1);
                write_paint_id(out, id);
            }
            None => out.push(0),
        }
    }
}

fn read_goal(cursor: &mut Cursor<'_>, string_ids: bool) -> Result<GoalSettings, SaveFormatError> {
    let material = read_material_id(cursor, string_ids)?;
    let facing = decode_settings_facing(cursor.read_u8()?)?;
    let mut stamps = [None; 4];
    for slot in &mut stamps {
        *slot = if cursor.read_u8()? == 1 {
            Some(read_stamp_id(cursor, string_ids)?)
        } else {
            None
        };
    }
    let mut paints = [None; 4];
    for slot in &mut paints {
        *slot = if cursor.read_u8()? == 1 {
            Some(read_paint_id(cursor, string_ids)?)
        } else {
            None
        };
    }
    Ok(GoalSettings {
        material,
        facing,
        stamps,
        paints,
    })
}

fn encode_settings_facing(facing: crate::game::world::direction::Facing) -> u8 {
    match facing {
        crate::game::world::direction::Facing::North => 0,
        crate::game::world::direction::Facing::East => 1,
        crate::game::world::direction::Facing::South => 2,
        crate::game::world::direction::Facing::West => 3,
    }
}

fn decode_settings_facing(
    value: u8,
) -> Result<crate::game::world::direction::Facing, SaveFormatError> {
    match value {
        0 => Ok(crate::game::world::direction::Facing::North),
        1 => Ok(crate::game::world::direction::Facing::East),
        2 => Ok(crate::game::world::direction::Facing::South),
        3 => Ok(crate::game::world::direction::Facing::West),
        _ => Err(SaveFormatError::InvalidFacing(value)),
    }
}

fn write_stamper(out: &mut Vec<u8>, settings: StamperSettings) {
    write_stamp_id(out, settings.stamp);
}

fn read_stamper(
    cursor: &mut Cursor<'_>,
    string_ids: bool,
) -> Result<StamperSettings, SaveFormatError> {
    Ok(StamperSettings {
        stamp: read_stamp_id(cursor, string_ids)?,
    })
}

fn write_roller(out: &mut Vec<u8>, settings: RollerSettings) {
    write_paint_id(out, settings.paint);
}

fn read_roller(
    cursor: &mut Cursor<'_>,
    string_ids: bool,
) -> Result<RollerSettings, SaveFormatError> {
    Ok(RollerSettings {
        paint: read_paint_id(cursor, string_ids)?,
    })
}

fn write_converter(out: &mut Vec<u8>, settings: ConverterSettings) {
    out.push(match settings.mode {
        ConverterMode::AnyInput => 0,
        ConverterMode::SpecificInput => 1,
    });
    write_material_id(out, settings.input);
    write_material_id(out, settings.output);
}

fn read_converter(
    cursor: &mut Cursor<'_>,
    string_ids: bool,
) -> Result<ConverterSettings, SaveFormatError> {
    let mode = match cursor.read_u8()? {
        0 => ConverterMode::AnyInput,
        1 => ConverterMode::SpecificInput,
        _ => return Err(SaveFormatError::InvalidSettings),
    };
    Ok(ConverterSettings {
        mode,
        input: read_material_id(cursor, string_ids)?,
        output: read_material_id(cursor, string_ids)?,
    })
}

fn write_teleport(out: &mut Vec<u8>, settings: &TeleportSettings) {
    write_string(out, &settings.name);
    if let Some(pair) = settings.pair {
        out.push(1);
        out.extend_from_slice(&pair.x.to_le_bytes());
        out.extend_from_slice(&pair.y.to_le_bytes());
        out.extend_from_slice(&pair.z.to_le_bytes());
    } else {
        out.push(0);
    }
}

fn read_teleport(cursor: &mut Cursor<'_>) -> Result<TeleportSettings, SaveFormatError> {
    let name = cursor.read_string()?;
    let pair = if cursor.read_u8()? == 1 {
        Some(IVec3::new(
            cursor.read_i32()?,
            cursor.read_i32()?,
            cursor.read_i32()?,
        ))
    } else {
        None
    };
    Ok(TeleportSettings { name, pair })
}

fn write_sign(out: &mut Vec<u8>, settings: &SignSettings) {
    match &settings.text {
        Some(text) => {
            out.push(1);
            write_string(out, text);
        }
        None => out.push(0),
    }
    match settings.display {
        Some(SignDisplay::Material(material)) => {
            out.push(1);
            write_material_id(out, material);
        }
        Some(SignDisplay::Stamp(stamp)) => {
            out.push(2);
            write_stamp_id(out, stamp);
        }
        None => out.push(0),
    }
}

fn read_sign(cursor: &mut Cursor<'_>, string_ids: bool) -> Result<SignSettings, SaveFormatError> {
    let text = if cursor.read_u8()? == 1 {
        Some(cursor.read_string()?)
    } else {
        None
    };
    let display = match cursor.read_u8()? {
        0 => None,
        1 => Some(SignDisplay::Material(read_material_id(cursor, string_ids)?)),
        2 => Some(SignDisplay::Stamp(read_stamp_id(cursor, string_ids)?)),
        _ => return Err(SaveFormatError::InvalidSettings),
    };
    Ok(SignSettings { text, display })
}

fn write_material_id(out: &mut Vec<u8>, id: MaterialBlockId) {
    ensure_fallback_material_catalog();
    let catalog = material_catalog();
    let string_id = catalog
        .string_id(id)
        .unwrap_or(oif_sim::blocks::FALLBACK_MATERIAL_STRING_ID);
    write_string(out, string_id);
}

fn read_material_id(
    cursor: &mut Cursor<'_>,
    string_ids: bool,
) -> Result<MaterialBlockId, SaveFormatError> {
    ensure_fallback_material_catalog();
    if string_ids {
        let string_id = cursor.read_string()?;
        return Ok(resolve_material_id(&string_id));
    }
    // 旧数字材料枚举已弃用，一律兜底
    let _ = cursor.read_u8()?;
    Ok(fallback_material_id())
}

fn write_stamp_id(out: &mut Vec<u8>, id: StampMaterialId) {
    ensure_fallback_stamp_catalog();
    let catalog = stamp_catalog();
    let string_id = catalog
        .string_id(id)
        .unwrap_or_else(|| panic!("unknown StampMaterialId {}", id.0));
    write_string(out, string_id);
}

fn read_stamp_id(
    cursor: &mut Cursor<'_>,
    string_ids: bool,
) -> Result<StampMaterialId, SaveFormatError> {
    ensure_fallback_stamp_catalog();
    if string_ids {
        let string_id = cursor.read_string()?;
        return stamp_id_by_string(&string_id)
            .ok_or(SaveFormatError::UnknownStampMaterialId(string_id));
    }
    Ok(match cursor.read_u8()? {
        0 => stamp_id_by_string("red").expect("fallback red"),
        1 => stamp_id_by_string("green").expect("fallback green"),
        2 => stamp_id_by_string("blue").expect("fallback blue"),
        3 => stamp_id_by_string("yellow").expect("fallback yellow"),
        _ => return Err(SaveFormatError::InvalidSettings),
    })
}

fn write_paint_id(out: &mut Vec<u8>, id: PaintMaterialId) {
    ensure_fallback_paint_catalog();
    let catalog = paint_catalog();
    let string_id = catalog
        .string_id(id)
        .unwrap_or_else(|| panic!("unknown PaintMaterialId {}", id.0));
    write_string(out, string_id);
}

fn read_paint_id(
    cursor: &mut Cursor<'_>,
    string_ids: bool,
) -> Result<PaintMaterialId, SaveFormatError> {
    ensure_fallback_paint_catalog();
    if string_ids {
        let string_id = cursor.read_string()?;
        return paint_id_by_string(&string_id)
            .ok_or(SaveFormatError::UnknownPaintMaterialId(string_id));
    }
    Ok(match cursor.read_u8()? {
        0 => paint_id_by_string("red").expect("fallback red"),
        1 => paint_id_by_string("green").expect("fallback green"),
        2 => paint_id_by_string("blue").expect("fallback blue"),
        3 => paint_id_by_string("yellow").expect("fallback yellow"),
        _ => return Err(SaveFormatError::InvalidSettings),
    })
}
