//! 方块设置类型与读写

use glam::IVec3;
use serde::{Deserialize, Serialize};

use crate::blocks::{
    MaterialBlockId, PaintMaterialId, StampMaterialId, fallback_material_id, paint_id_by_string,
    stamp_id_by_string,
};
use crate::world::direction::Facing;

use super::WorldBlocks;

/// 方块设置：按种类存生成器/验收/印花机等参数
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BlockSettings {
    Generator(GeneratorSettings),
    Goal(GoalSettings),
    Stamper(StamperSettings),
    Roller(RollerSettings),
    Converter(ConverterSettings),
    Teleport(TeleportSettings),
    Sign(SignSettings),
}

impl BlockSettings {
    fn matches_kind(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Generator(_), Self::Generator(_))
                | (Self::Goal(_), Self::Goal(_))
                | (Self::Stamper(_), Self::Stamper(_))
                | (Self::Roller(_), Self::Roller(_))
                | (Self::Converter(_), Self::Converter(_))
                | (Self::Teleport(_), Self::Teleport(_))
                | (Self::Sign(_), Self::Sign(_))
        )
    }
}

/// 生成器触发模式：周期或连接验收结构（Link 存代表 Goal 坐标，加载时由连通性解析）
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum GeneratorMode {
    Period { period: u64, offset: u64 },
    Link { anchor: Option<IVec3> },
}

/// 生成器设置：触发模式、材料与朝向
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeneratorSettings {
    pub mode: GeneratorMode,
    pub material: MaterialBlockId,
    /// 有向材料的生成朝向（非有向材料忽略）
    pub facing: Facing,
}

impl GeneratorSettings {
    /// 同参相连判定键（忽略材料种类）
    pub fn trigger_key(self) -> GeneratorMode {
        match self.mode {
            GeneratorMode::Period { period, offset } => GeneratorMode::Period {
                period: period.max(1),
                offset: offset % period.max(1),
            },
            link => link,
        }
    }

    pub fn clamps_offset(mut self) -> Self {
        if let GeneratorMode::Period { period, offset } = &mut self.mode {
            let p = (*period).max(1);
            *period = p;
            *offset %= p;
        }
        self
    }
}

/// 验收器设置：目标材料、朝向与印花/漆要求
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GoalSettings {
    pub material: MaterialBlockId,
    /// 有向材料的验收朝向（非有向材料忽略）
    pub facing: Facing,
    /// 要求附着的印花（空槽忽略；有值则精确匹配多重集合）
    pub stamps: [Option<StampMaterialId>; 4],
    /// 要求附着的漆（空槽忽略；有值则精确匹配多重集合）
    pub paints: [Option<PaintMaterialId>; 4],
}

/// 印花机设置：选用的印花材料
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StamperSettings {
    pub stamp: StampMaterialId,
}

/// 滚刷设置：选用的漆材料
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RollerSettings {
    pub paint: PaintMaterialId,
}

/// 告示牌展示图标：材料或印花（与文本互斥）
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SignDisplay {
    Material(MaterialBlockId),
    Stamp(StampMaterialId),
}

/// 告示牌设置：文本或图标二选一
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SignSettings {
    pub text: Option<String>,
    pub display: Option<SignDisplay>,
}

/// 转换器设置：输入/输出材料与模式
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConverterSettings {
    pub mode: ConverterMode,
    pub input: MaterialBlockId,
    pub output: MaterialBlockId,
}

impl Default for ConverterSettings {
    fn default() -> Self {
        let fallback = fallback_material_id();
        Self {
            mode: ConverterMode::AnyInput,
            input: fallback,
            output: fallback,
        }
    }
}

/// 转换器输入匹配：任意输入或指定输入
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConverterMode {
    AnyInput,
    SpecificInput,
}

/// 传送门设置：显示名与配对坐标
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TeleportSettings {
    pub name: String,
    pub pair: Option<IVec3>,
}

impl TeleportSettings {
    pub fn unnamed(pos: IVec3) -> Self {
        Self {
            name: format!("Portal {}", pos_hash(pos)),
            pair: None,
        }
    }
}

impl Default for StamperSettings {
    fn default() -> Self {
        Self {
            stamp: stamp_id_by_string("red").expect("fallback red stamp"),
        }
    }
}

impl Default for RollerSettings {
    fn default() -> Self {
        Self {
            paint: paint_id_by_string("red").expect("fallback red paint"),
        }
    }
}

impl Default for GoalSettings {
    fn default() -> Self {
        Self {
            material: fallback_material_id(),
            facing: Facing::North,
            stamps: [None; 4],
            paints: [None; 4],
        }
    }
}

impl Default for GeneratorSettings {
    fn default() -> Self {
        Self {
            mode: GeneratorMode::Period {
                period: crate::blocks::DEFAULT_GENERATOR_PERIOD,
                offset: 0,
            },
            material: fallback_material_id(),
            facing: Facing::North,
        }
    }
}

fn pos_hash(pos: IVec3) -> i32 {
    pos.x.abs() * 31 + pos.y.abs() * 17 + pos.z.abs() * 13
}

impl WorldBlocks {
    pub fn generator_settings(&self, pos: IVec3) -> GeneratorSettings {
        match self.block_settings.get(&pos) {
            Some(BlockSettings::Generator(settings)) => *settings,
            _ => GeneratorSettings::default(),
        }
    }

    pub fn set_block_settings(&mut self, pos: IVec3, settings: BlockSettings) {
        let block = self
            .system_blocks
            .get(&pos)
            .copied()
            .or_else(|| self.blocks.get(&pos).copied());
        let Some(block) = block else {
            return;
        };
        let Some(default_settings) = block.kind.default_settings(pos) else {
            return;
        };
        if !settings.matches_kind(&default_settings) {
            return;
        }
        if self.block_settings.get(&pos) != Some(&settings) {
            self.block_settings.insert(pos, settings);
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }

    pub fn sign_settings(&self, pos: IVec3) -> SignSettings {
        match self.block_settings.get(&pos) {
            Some(BlockSettings::Sign(settings)) => settings.clone(),
            _ => SignSettings::default(),
        }
    }

    pub fn set_sign_settings(&mut self, pos: IVec3, settings: SignSettings) {
        self.set_block_settings(pos, BlockSettings::Sign(settings));
    }

    pub fn set_generator_settings(&mut self, pos: IVec3, settings: GeneratorSettings) {
        self.set_block_settings(pos, BlockSettings::Generator(settings.clamps_offset()));
    }

    pub fn goal_settings(&self, pos: IVec3) -> GoalSettings {
        match self.block_settings.get(&pos) {
            Some(BlockSettings::Goal(settings)) => *settings,
            _ => GoalSettings::default(),
        }
    }

    pub fn set_goal_settings(&mut self, pos: IVec3, settings: GoalSettings) {
        self.set_block_settings(pos, BlockSettings::Goal(settings));
    }

    pub fn stamper_settings(&self, pos: IVec3) -> StamperSettings {
        match self.block_settings.get(&pos) {
            Some(BlockSettings::Stamper(settings)) => *settings,
            _ => StamperSettings::default(),
        }
    }

    pub fn set_stamper_settings(&mut self, pos: IVec3, settings: StamperSettings) {
        self.set_block_settings(pos, BlockSettings::Stamper(settings));
    }

    pub fn roller_settings(&self, pos: IVec3) -> RollerSettings {
        match self.block_settings.get(&pos) {
            Some(BlockSettings::Roller(settings)) => *settings,
            _ => RollerSettings::default(),
        }
    }

    pub fn set_roller_settings(&mut self, pos: IVec3, settings: RollerSettings) {
        self.set_block_settings(pos, BlockSettings::Roller(settings));
    }

    pub fn converter_settings(&self, pos: IVec3) -> ConverterSettings {
        match self.block_settings.get(&pos) {
            Some(BlockSettings::Converter(settings)) => *settings,
            _ => ConverterSettings::default(),
        }
    }

    pub fn set_converter_settings(&mut self, pos: IVec3, settings: ConverterSettings) {
        self.set_block_settings(pos, BlockSettings::Converter(settings));
    }
}
