use bevy::prelude::*;

use crate::game::blocks::{BlockKind, MaterialKind, StampColor};
use crate::game::world::grid::{
    BlockSettings, ConverterMode, ConverterSettings, GeneratorMode, GeneratorSettings, GoalSettings,
    LabelerSettings, TeleportSettings,
};

use super::Cursor;
use super::SaveFormatError;

pub fn write_settings(out: &mut Vec<u8>, kind: BlockKind, settings: &BlockSettings) {
    match (kind, settings) {
        (BlockKind::Generator, BlockSettings::Generator(value)) => write_generator(out, *value),
        (BlockKind::Goal, BlockSettings::Goal(value)) => write_goal(out, *value),
        (BlockKind::Stamper | BlockKind::Roller, BlockSettings::Labeler(value)) => {
            write_labeler(out, *value)
        }
        (BlockKind::Converter, BlockSettings::Converter(value)) => write_converter(out, *value),
        (
            BlockKind::TeleportEntrance | BlockKind::TeleportExit,
            BlockSettings::Teleport(value),
        ) => write_teleport(out, value),
        _ => {}
    }
}

pub fn read_settings(
    cursor: &mut Cursor<'_>,
    kind: BlockKind,
) -> Result<BlockSettings, SaveFormatError> {
    Ok(match kind {
        BlockKind::Generator => BlockSettings::Generator(read_generator(cursor)?),
        BlockKind::Goal => BlockSettings::Goal(read_goal(cursor)?),
        BlockKind::Stamper | BlockKind::Roller => BlockSettings::Labeler(read_labeler(cursor)?),
        BlockKind::Converter => BlockSettings::Converter(read_converter(cursor)?),
        BlockKind::TeleportEntrance | BlockKind::TeleportExit => {
            BlockSettings::Teleport(read_teleport(cursor)?)
        }
        _ => return Err(SaveFormatError::InvalidSettings),
    })
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
    out.push(encode_material(settings.material));
}

fn read_generator(cursor: &mut Cursor<'_>) -> Result<GeneratorSettings, SaveFormatError> {
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
        material: decode_material(cursor.read_u8()?)?,
    })
}

fn write_goal(out: &mut Vec<u8>, settings: GoalSettings) {
    out.push(encode_material(settings.material));
}

fn read_goal(cursor: &mut Cursor<'_>) -> Result<GoalSettings, SaveFormatError> {
    Ok(GoalSettings {
        material: decode_material(cursor.read_u8()?)?,
    })
}

fn write_labeler(out: &mut Vec<u8>, settings: LabelerSettings) {
    out.push(encode_stamp_color(settings.color));
}

fn read_labeler(cursor: &mut Cursor<'_>) -> Result<LabelerSettings, SaveFormatError> {
    Ok(LabelerSettings {
        color: decode_stamp_color(cursor.read_u8()?)?,
    })
}

fn write_converter(out: &mut Vec<u8>, settings: ConverterSettings) {
    out.push(match settings.mode {
        ConverterMode::AnyInput => 0,
        ConverterMode::SpecificInput => 1,
    });
    out.push(encode_material(settings.input));
    out.push(encode_material(settings.output));
}

fn read_converter(cursor: &mut Cursor<'_>) -> Result<ConverterSettings, SaveFormatError> {
    let mode = match cursor.read_u8()? {
        0 => ConverterMode::AnyInput,
        1 => ConverterMode::SpecificInput,
        _ => return Err(SaveFormatError::InvalidSettings),
    };
    Ok(ConverterSettings {
        mode,
        input: decode_material(cursor.read_u8()?)?,
        output: decode_material(cursor.read_u8()?)?,
    })
}

fn write_teleport(out: &mut Vec<u8>, settings: &TeleportSettings) {
    let name = settings.name.as_bytes();
    let len = name.len().min(u16::MAX as usize) as u16;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(&name[..len as usize]);
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
    let len = cursor.read_u16()? as usize;
    let name_bytes = cursor.read_bytes(len)?;
    let name = std::str::from_utf8(name_bytes)
        .map(|s| s.to_string())
        .map_err(|_| SaveFormatError::InvalidSettings)?;
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

fn encode_material(material: MaterialKind) -> u8 {
    match material {
        MaterialKind::Basic => 0,
        MaterialKind::Iron => 1,
        MaterialKind::Copper => 2,
    }
}

fn decode_material(value: u8) -> Result<MaterialKind, SaveFormatError> {
    Ok(match value {
        0 => MaterialKind::Basic,
        1 => MaterialKind::Iron,
        2 => MaterialKind::Copper,
        _ => return Err(SaveFormatError::InvalidSettings),
    })
}

fn encode_stamp_color(color: StampColor) -> u8 {
    match color {
        StampColor::Red => 0,
        StampColor::Green => 1,
        StampColor::Blue => 2,
        StampColor::Yellow => 3,
    }
}

fn decode_stamp_color(value: u8) -> Result<StampColor, SaveFormatError> {
    Ok(match value {
        0 => StampColor::Red,
        1 => StampColor::Green,
        2 => StampColor::Blue,
        3 => StampColor::Yellow,
        _ => return Err(SaveFormatError::InvalidSettings),
    })
}
