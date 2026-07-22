use bevy::prelude::*;
use oif_sim::blocks::{BlockData, BlockKind, PersistentLayer};
use oif_sim::world::grid::{BlockSettings, MaterialFace, WorldBlocks};
use serde::{Deserialize, Serialize};

use crate::shared::persistent_storage;
use crate::shared::save_format::{
    self, BLOCKS_FILE, META_FILE, SKYBOX_FILE, SaveBlocksData, SavedBlock,
};

/// 存档快捷栏格数（与 UI 快捷栏一致）
pub const HOTBAR_SLOTS: usize = 9;

/// 存档中的区域工具种类（纯 DTO，与 UI AreaKind 对应）
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SavedAreaKind {
    Selection,
}

/// 存档快捷栏物品（纯 DTO，与 UI InventoryItem 对应）
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SavedHotbarItem {
    Block(BlockKind),
    Area(SavedAreaKind),
    /// 灯面板：贴在电线表面，不占邻格
    LightPanel,
}

/// 存档快捷栏 9 格
pub type SavedHotbar = [Option<SavedHotbarItem>; HOTBAR_SLOTS];

const SAVE_VERSION: u32 = 1;

/// 新建谜题时写入的默认天空盒（模板缺失时的兜底）
const DEFAULT_SKYBOX_PNG: &[u8] = include_bytes!("../../assets/skybox.png");

/// 默认新存档模板（嵌入；桌面也可改 assets/save_templates/default/）
const TEMPLATE_META_JSON: &str = include_str!("../../assets/save_templates/default/meta.json");
const TEMPLATE_BLOCKS_BIN: &[u8] = include_bytes!("../../assets/save_templates/default/blocks.bin");
const TEMPLATE_SKYBOX_PNG: &[u8] = include_bytes!("../../assets/save_templates/default/skybox.png");

/// 存档寻址：Puzzle 为顶层目录，Solution 在其 `solutions/` 子目录下
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SaveSlot {
    pub puzzle: String,
    pub solution: Option<String>,
}

impl SaveSlot {
    pub fn puzzle(name: impl Into<String>) -> Self {
        Self {
            puzzle: normalized_save_name(&name.into()),
            solution: None,
        }
    }

    pub fn solution(puzzle: impl Into<String>, solution: impl Into<String>) -> Self {
        Self {
            puzzle: normalized_save_name(&puzzle.into()),
            solution: Some(normalized_save_name(&solution.into())),
        }
    }

    pub fn kind(&self) -> SaveKind {
        if self.solution.is_some() {
            SaveKind::Solution
        } else {
            SaveKind::Puzzle
        }
    }

    pub fn storage_path(&self) -> String {
        match &self.solution {
            None => sanitize_save_name(&self.puzzle),
            Some(solution) => format!(
                "{}/solutions/{}",
                sanitize_save_name(&self.puzzle),
                sanitize_save_name(solution)
            ),
        }
    }

    pub fn display_name(&self) -> String {
        read_save_name(self).unwrap_or_else(|| {
            self.solution
                .as_ref()
                .cloned()
                .unwrap_or_else(|| self.puzzle.clone())
        })
    }

    pub fn from_storage_path(path: &str) -> Option<Self> {
        let parts: Vec<&str> = path.split('/').collect();
        match parts.as_slice() {
            [puzzle] => Some(Self::puzzle(*puzzle)),
            [puzzle, "solutions", solution] if !puzzle.is_empty() && !solution.is_empty() => {
                Some(Self::solution(*puzzle, *solution))
            }
            _ => None,
        }
    }
}

#[derive(Resource, Default)]
pub struct SaveState {
    pub current: Option<SaveSlot>,
    pub current_kind: Option<SaveKind>,
    pub entries: Vec<SaveEntry>,
    pub selected_puzzle: Option<String>,
    selected_puzzle_solutions: Vec<SaveEntry>,
}

impl SaveState {
    pub fn refresh(&mut self) {
        self.entries = list_save_entries();
        self.refresh_selected_puzzle_solutions();
    }

    pub fn puzzles(&self) -> Vec<&SaveEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.kind == SaveKind::Puzzle)
            .collect()
    }

    pub fn select_puzzle(&mut self, puzzle: Option<String>) {
        if self.selected_puzzle == puzzle {
            return;
        }
        self.selected_puzzle = puzzle;
        self.refresh_selected_puzzle_solutions();
    }

    pub fn selected_puzzle_solutions(&self) -> &[SaveEntry] {
        &self.selected_puzzle_solutions
    }

    fn refresh_selected_puzzle_solutions(&mut self) {
        self.selected_puzzle_solutions = match self.selected_puzzle.as_deref() {
            Some(puzzle) => self
                .entries
                .iter()
                .filter(|entry| entry.kind == SaveKind::Solution && entry.slot.puzzle == puzzle)
                .cloned()
                .collect(),
            None => Vec::new(),
        };
    }
}

#[derive(Clone)]
pub struct SaveEntry {
    pub slot: SaveSlot,
    pub name: String,
    pub kind: SaveKind,
}

impl SaveEntry {
    pub fn puzzle_id(&self) -> Option<&str> {
        (self.kind == SaveKind::Solution).then_some(self.slot.puzzle.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SaveKind {
    Puzzle,
    Solution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SaveMetaKind {
    Puzzle,
    Solution,
}

#[derive(Serialize, Deserialize)]
struct SaveMeta {
    version: u32,
    kind: SaveMetaKind,
    /// 存档名字（可中文）；缺省时列表用文件夹名兜底
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(default)]
    puzzle_id: Option<String>,
    #[serde(default)]
    hotbar: Option<SavedHotbar>,
    #[serde(default)]
    player: Option<PlayerSave>,
    /// 平行光与天空光照；字段见 `schemas/save.meta.schema.json`（存档勿写 `$schema`）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sun: Option<SunMeta>,
    /// 环境光；省略则用内置默认
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ambient: Option<AmbientMeta>,
    /// 旧字段：仅方向；若同时有 `sun` 则以 `sun` 为准，否则并入 `sun.direction`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sun_direction: Option<[f32; 3]>,
}

/// meta.json 里的平行光 / 天空盒亮度配置（各项均可选）
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct SunMeta {
    /// 光线前进方向（太阳 → 地面）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    direction: Option<[f32; 3]>,
    /// 照度（lux），默认约 9500
    #[serde(default, skip_serializing_if = "Option::is_none")]
    illuminance: Option<f32>,
    /// 直接 sRGB 颜色 [r,g,b]；若同时写了 color_temperature，以 color 为准
    #[serde(default, skip_serializing_if = "Option::is_none")]
    color: Option<[f32; 3]>,
    /// 色温（开尔文），例如 5500；无 color 时换算为颜色
    #[serde(default, skip_serializing_if = "Option::is_none")]
    color_temperature: Option<f32>,
    /// Bevy 贴图天空盒亮度，默认 1000
    #[serde(default, skip_serializing_if = "Option::is_none")]
    skybox_brightness: Option<f32>,
}

/// meta.json 里的环境光配置
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct AmbientMeta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    color: Option<[f32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    brightness: Option<f32>,
}

/// 加载后解析好的谜题光照（缺省项已填默认值）
#[derive(Resource, Clone, Copy, Debug)]
pub struct PuzzleLighting {
    pub direction: Option<Vec3>,
    pub illuminance: f32,
    pub color: Color,
    pub skybox_brightness: f32,
    pub ambient_color: Color,
    pub ambient_brightness: f32,
}

impl Default for PuzzleLighting {
    fn default() -> Self {
        Self {
            direction: None,
            illuminance: 9500.0,
            color: Color::srgb(1.0, 0.97, 0.92),
            skybox_brightness: 1000.0,
            ambient_color: Color::srgb(0.90, 0.94, 1.0),
            ambient_brightness: 680.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlayerSave {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub flying: bool,
}

struct SaveFile {
    meta: SaveMeta,
    blocks: SaveBlocksData,
}

pub fn save_puzzle(
    world: &WorldBlocks,
    slot: &SaveSlot,
    hotbar: &SavedHotbar,
    player: Option<PlayerSave>,
) -> bool {
    if slot.solution.is_some() {
        return false;
    }
    write_save(
        slot,
        SaveFile::puzzle(capture_puzzle_layer(world, hotbar), player),
    )
}

/// 按名字另存为新谜题（创建时写入名字）
pub fn save_puzzle_as(
    world: &WorldBlocks,
    name: &str,
    hotbar: &SavedHotbar,
    player: Option<PlayerSave>,
) -> Option<SaveSlot> {
    let slot = allocate_puzzle_slot(name)?;
    let mut save = SaveFile::puzzle(capture_puzzle_layer(world, hotbar), player);
    save.meta.name = Some(name.trim().to_string());
    write_save(&slot, save).then_some(slot)
}

/// 从默认模板新建谜题；成功返回槽位（创建时写入名字）
pub fn create_puzzle_from_default_template(name: &str) -> Option<SaveSlot> {
    let slot = allocate_puzzle_slot(name)?;
    let path = slot.storage_path();
    let meta_text = load_template_text(META_FILE).unwrap_or_else(|| TEMPLATE_META_JSON.to_string());
    let meta = match serde_json::from_str::<SaveMeta>(&meta_text) {
        Ok(mut meta) => {
            meta.name = Some(name.trim().to_string());
            match serde_json::to_string_pretty(&meta) {
                Ok(serialized) => serialized,
                Err(error) => {
                    warn!("Failed to serialize template meta: {error}");
                    return None;
                }
            }
        }
        Err(error) => {
            warn!("Failed to parse template meta: {error}");
            return None;
        }
    };
    let blocks = load_template_bytes(BLOCKS_FILE).unwrap_or_else(|| TEMPLATE_BLOCKS_BIN.to_vec());
    let skybox = load_template_bytes(SKYBOX_FILE).unwrap_or_else(|| TEMPLATE_SKYBOX_PNG.to_vec());
    if !persistent_storage::write_save_text(&path, META_FILE, &meta) {
        return None;
    }
    if !persistent_storage::write_save_bytes(&path, BLOCKS_FILE, &blocks) {
        return None;
    }
    if !persistent_storage::write_save_bytes(&path, SKYBOX_FILE, &skybox) {
        return None;
    }
    Some(slot)
}

/// 按名字新建通关存档（创建时写入名字）
pub fn save_solution_as(
    world: &WorldBlocks,
    puzzle: &str,
    name: &str,
    hotbar: &SavedHotbar,
    player: Option<PlayerSave>,
) -> Option<SaveSlot> {
    let slot = allocate_solution_slot(puzzle, name)?;
    let mut save = SaveFile::solution(
        puzzle,
        capture_factory_blocks(world),
        capture_wire_face_panels(world),
        hotbar,
        player,
    );
    save.meta.name = Some(name.trim().to_string());
    write_save(&slot, save).then_some(slot)
}

/// 改名：更新 meta 中的名字；文件夹仅在 sanitize 后可用时跟着改
pub fn rename_save_to(old: &SaveSlot, name: &str) -> Option<SaveSlot> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let sanitized = normalized_save_name(name);
    let mut slot = old.clone();
    if !sanitized.is_empty() {
        let candidate = match old.kind() {
            SaveKind::Puzzle => SaveSlot::puzzle(&sanitized),
            SaveKind::Solution => SaveSlot::solution(&old.puzzle, &sanitized),
        };
        if candidate.storage_path() != old.storage_path()
            && !persistent_storage::save_exists(&candidate.storage_path())
            && rename_save_folder(old, &candidate)
        {
            slot = candidate;
        }
    }
    let Some(mut save) = read_save(&slot) else {
        return None;
    };
    save.meta.name = Some(name.to_string());
    let path = slot.storage_path();
    let meta = match serde_json::to_string_pretty(&save.meta) {
        Ok(serialized) => serialized,
        Err(error) => {
            warn!("Failed to serialize save meta: {error}");
            return None;
        }
    };
    if !persistent_storage::write_save_text(&path, META_FILE, &meta) {
        return None;
    }
    Some(slot)
}

fn allocate_puzzle_slot(name: &str) -> Option<SaveSlot> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let storage = next_named_save(&persistent_storage::list_puzzles(), name);
    if storage.is_empty() {
        return None;
    }
    let slot = SaveSlot::puzzle(&storage);
    if persistent_storage::save_exists(&slot.storage_path()) {
        return None;
    }
    Some(slot)
}

fn allocate_solution_slot(puzzle: &str, name: &str) -> Option<SaveSlot> {
    let name = name.trim();
    if name.is_empty() || puzzle.is_empty() {
        return None;
    }
    let storage = next_named_save(&persistent_storage::list_solution_names(puzzle), name);
    if storage.is_empty() {
        return None;
    }
    let slot = SaveSlot::solution(puzzle, &storage);
    if persistent_storage::save_exists(&slot.storage_path()) {
        return None;
    }
    Some(slot)
}

/// 读 assets/save_templates/default/ 下文本（失败则无）
fn load_template_text(file: &str) -> Option<String> {
    let bytes = load_template_bytes(file)?;
    String::from_utf8(bytes).ok()
}

/// 读 assets/save_templates/default/ 下字节；失败则用嵌入常量（wasm 等）
fn load_template_bytes(file: &str) -> Option<Vec<u8>> {
    let path = std::path::PathBuf::from(crate::shared::platform::asset_path())
        .join("save_templates")
        .join("default")
        .join(file);
    if let Ok(bytes) = crate::shared::asset_io::read_bytes(&path) {
        return Some(bytes);
    }
    match file {
        META_FILE => Some(TEMPLATE_META_JSON.as_bytes().to_vec()),
        BLOCKS_FILE => Some(TEMPLATE_BLOCKS_BIN.to_vec()),
        SKYBOX_FILE => Some(TEMPLATE_SKYBOX_PNG.to_vec()),
        _ => None,
    }
}

pub fn save_solution(
    world: &WorldBlocks,
    slot: &SaveSlot,
    hotbar: &SavedHotbar,
    player: Option<PlayerSave>,
) -> bool {
    let Some(solution) = slot.solution.as_deref() else {
        return false;
    };
    if solution.is_empty() {
        return false;
    }
    write_save(
        slot,
        SaveFile::solution(
            &slot.puzzle,
            capture_factory_blocks(world),
            capture_wire_face_panels(world),
            hotbar,
            player,
        ),
    )
}

pub fn load_world(world: &mut WorldBlocks, slot: &SaveSlot) -> Option<LoadedSave> {
    let loaded = decode_save_slot(slot)?;
    *world = loaded.world.clone();
    Some(loaded)
}

/// 仅解码存档（可在后台任务中跑，不碰 ECS）
pub fn decode_save_slot(slot: &SaveSlot) -> Option<LoadedSave> {
    let save = read_save(slot)?;
    save.into_loaded(slot)
}

pub fn save_kind(slot: &SaveSlot) -> Option<SaveKind> {
    let save = read_save(slot)?;
    Some(save.meta_kind())
}

pub fn has_solutions_for_puzzle(puzzle: &str) -> bool {
    !persistent_storage::list_solution_names(puzzle).is_empty()
}

pub fn invalidate_solutions_for_puzzle(puzzle: &str) -> usize {
    persistent_storage::list_solution_names(puzzle)
        .into_iter()
        .filter(|name| delete_save(&SaveSlot::solution(puzzle, name)))
        .count()
}

pub fn delete_save(slot: &SaveSlot) -> bool {
    persistent_storage::remove_save_folder(&slot.storage_path())
}

pub fn rename_save_folder(old: &SaveSlot, new: &SaveSlot) -> bool {
    if old.kind() != new.kind() {
        return false;
    }
    if old.solution.is_some() && old.puzzle != new.puzzle {
        return false;
    }
    let old_path = old.storage_path();
    let new_path = new.storage_path();
    if old_path == new_path {
        return true;
    }
    if new.puzzle.is_empty() || new.solution.as_deref().is_some_and(str::is_empty) {
        return false;
    }
    if persistent_storage::save_exists(&new_path) {
        warn!("Cannot rename save {old_path} to {new_path}: target already exists");
        return false;
    }
    if !persistent_storage::save_exists(&old_path) {
        warn!("Cannot rename save {old_path}: source missing");
        return false;
    }
    persistent_storage::rename_save_folder(&old_path, &new_path)
}

pub fn reset_solution_world(world: &mut WorldBlocks, puzzle_snapshot: &WorldBlocks) {
    *world = puzzle_snapshot.clone();
}

#[derive(Clone)]
pub struct LoadedSave {
    pub world: WorldBlocks,
    pub puzzle_snapshot: Option<WorldBlocks>,
    pub puzzle_id: Option<String>,
    pub hotbar: Option<SavedHotbar>,
    pub player: Option<PlayerSave>,
    /// 谜题光照（来自 puzzle meta；solution 读所属 puzzle）
    pub lighting: PuzzleLighting,
}

impl SaveFile {
    fn puzzle(layer: PuzzleLayer, player: Option<PlayerSave>) -> Self {
        Self {
            meta: SaveMeta {
                version: SAVE_VERSION,
                kind: SaveMetaKind::Puzzle,
                name: None,
                puzzle_id: None,
                hotbar: layer.hotbar,
                player,
                sun: None,
                ambient: None,
                sun_direction: None,
            },
            blocks: SaveBlocksData {
                scene_blocks: layer.scene_blocks,
                system_blocks: layer.system_blocks,
                factory_blocks: Vec::new(),
                wire_face_panels: Vec::new(),
            },
        }
    }

    fn solution(
        puzzle_id: &str,
        factory_blocks: Vec<SavedBlock>,
        wire_face_panels: Vec<save_format::SavedWireFacePanel>,
        hotbar: &SavedHotbar,
        player: Option<PlayerSave>,
    ) -> Self {
        Self {
            meta: SaveMeta {
                version: SAVE_VERSION,
                kind: SaveMetaKind::Solution,
                name: None,
                puzzle_id: Some(puzzle_id.to_string()),
                hotbar: Some(*hotbar),
                player,
                sun: None,
                ambient: None,
                sun_direction: None,
            },
            blocks: SaveBlocksData {
                factory_blocks,
                wire_face_panels,
                ..Default::default()
            },
        }
    }

    fn meta_kind(&self) -> SaveKind {
        match self.meta.kind {
            SaveMetaKind::Solution => SaveKind::Solution,
            SaveMetaKind::Puzzle => SaveKind::Puzzle,
        }
    }

    fn into_loaded(self, slot: &SaveSlot) -> Option<LoadedSave> {
        match self.meta.kind {
            SaveMetaKind::Solution => {
                if slot.solution.is_none() {
                    return None;
                }
                let puzzle_id = slot.puzzle.clone();
                if self
                    .meta
                    .puzzle_id
                    .as_deref()
                    .filter(|id| !id.is_empty())
                    .is_some_and(|id| id != puzzle_id)
                {
                    warn!("Solution save path puzzle `{puzzle_id}` disagrees with meta puzzle_id");
                    return None;
                }
                // 缺省快捷栏由 game 层按 BuilderMode 填默认
                let hotbar = self.meta.hotbar;
                let puzzle_world = load_puzzle_world(&puzzle_id)?;
                let lighting = read_puzzle_lighting(&puzzle_id);
                let mut world = puzzle_world.clone();
                apply_factory_blocks(&mut world, self.blocks.factory_blocks);
                apply_wire_face_panels(&mut world, self.blocks.wire_face_panels);
                Some(LoadedSave {
                    world,
                    puzzle_snapshot: Some(puzzle_world),
                    puzzle_id: Some(puzzle_id),
                    hotbar,
                    player: self.meta.player,
                    lighting,
                })
            }
            SaveMetaKind::Puzzle => {
                if slot.solution.is_some() {
                    return None;
                }
                let lighting = resolve_lighting(&self.meta);
                let layer = PuzzleLayer::from_blocks(self.blocks, self.meta.hotbar);
                // 缺省快捷栏由 game 层按 BuilderMode 填默认
                let hotbar = layer.hotbar;
                let mut world = WorldBlocks::default();
                apply_layer(&mut world, layer);
                Some(LoadedSave {
                    world,
                    puzzle_snapshot: None,
                    puzzle_id: None,
                    hotbar,
                    player: self.meta.player,
                    lighting,
                })
            }
        }
    }
}

struct PuzzleLayer {
    scene_blocks: Vec<SavedBlock>,
    system_blocks: Vec<SavedBlock>,
    hotbar: Option<SavedHotbar>,
}

impl PuzzleLayer {
    fn from_blocks(blocks: SaveBlocksData, hotbar: Option<SavedHotbar>) -> Self {
        Self {
            scene_blocks: blocks.scene_blocks,
            system_blocks: blocks.system_blocks,
            hotbar,
        }
    }
}

fn load_puzzle_world(puzzle: &str) -> Option<WorldBlocks> {
    let slot = SaveSlot::puzzle(puzzle);
    let save = read_save(&slot)?;
    if !matches!(save.meta.kind, SaveMetaKind::Puzzle) {
        return None;
    }
    let mut world = WorldBlocks::default();
    apply_layer(
        &mut world,
        PuzzleLayer::from_blocks(save.blocks, save.meta.hotbar),
    );
    Some(world)
}

fn write_save(slot: &SaveSlot, mut save: SaveFile) -> bool {
    // 覆盖保存：名字 / 光照只保留磁盘上已有的，不在保存路径改写
    if let Some(existing) = read_save(slot) {
        save.meta.name = existing.meta.name;
        save.meta.sun = existing.meta.sun;
        save.meta.ambient = existing.meta.ambient;
        save.meta.sun_direction = existing.meta.sun_direction;
    }
    let path = slot.storage_path();
    let meta = match serde_json::to_string_pretty(&save.meta) {
        Ok(serialized) => serialized,
        Err(error) => {
            warn!("Failed to serialize save meta: {error}");
            return false;
        }
    };
    if !persistent_storage::write_save_text(&path, META_FILE, &meta) {
        return false;
    }
    let blocks = save_format::encode_blocks(&save.blocks);
    if !persistent_storage::write_save_bytes(&path, BLOCKS_FILE, &blocks) {
        return false;
    }
    // 谜题存档：缺省时填入默认天空盒
    ensure_default_skybox_for_puzzle(&slot.puzzle);
    true
}

/// 读谜题 meta 光照
fn read_puzzle_lighting(puzzle: &str) -> PuzzleLighting {
    read_save(&SaveSlot::puzzle(puzzle))
        .map(|save| resolve_lighting(&save.meta))
        .unwrap_or_default()
}

/// 把 meta 里的 sun / ambient / 旧 sun_direction 合成运行时配置
fn resolve_lighting(meta: &SaveMeta) -> PuzzleLighting {
    let mut lighting = PuzzleLighting::default();
    let sun = meta.sun.clone().unwrap_or_default();
    let direction = sun
        .direction
        .or(meta.sun_direction)
        .and_then(parse_direction);
    lighting.direction = direction;
    if let Some(lux) = sun.illuminance {
        if lux.is_finite() && lux >= 0.0 {
            lighting.illuminance = lux;
        }
    }
    if let Some(rgb) = sun.color.and_then(parse_rgb) {
        lighting.color = Color::srgb(rgb[0], rgb[1], rgb[2]);
    } else if let Some(kelvin) = sun.color_temperature {
        if kelvin.is_finite() {
            let [r, g, b] = kelvin_to_srgb(kelvin);
            lighting.color = Color::srgb(r, g, b);
        }
    }
    if let Some(b) = sun.skybox_brightness {
        if b.is_finite() && b >= 0.0 {
            lighting.skybox_brightness = b;
        }
    }
    if let Some(ambient) = &meta.ambient {
        if let Some(rgb) = ambient.color.and_then(parse_rgb) {
            lighting.ambient_color = Color::srgb(rgb[0], rgb[1], rgb[2]);
        }
        if let Some(b) = ambient.brightness {
            if b.is_finite() && b >= 0.0 {
                lighting.ambient_brightness = b;
            }
        }
    }
    lighting
}

fn parse_direction(raw: [f32; 3]) -> Option<Vec3> {
    let v = Vec3::new(raw[0], raw[1], raw[2]);
    let n = v.normalize_or_zero();
    (n != Vec3::ZERO).then_some(n)
}

fn parse_rgb(raw: [f32; 3]) -> Option<[f32; 3]> {
    if raw.iter().all(|c| c.is_finite()) {
        Some([
            raw[0].clamp(0.0, 8.0),
            raw[1].clamp(0.0, 8.0),
            raw[2].clamp(0.0, 8.0),
        ])
    } else {
        None
    }
}

/// 色温（K）→ 近似 sRGB（Tanner Helland）
fn kelvin_to_srgb(kelvin: f32) -> [f32; 3] {
    let temp = (kelvin.clamp(1000.0, 40000.0) / 100.0) as f64;
    let (r, g, b) = if temp <= 66.0 {
        let r = 255.0;
        let g = (99.4708025861 * temp.ln() - 161.1195681661).clamp(0.0, 255.0);
        let b = if temp <= 19.0 {
            0.0
        } else {
            (138.5177312231 * (temp - 10.0).ln() - 305.0447927307).clamp(0.0, 255.0)
        };
        (r, g, b)
    } else {
        let r = (329.698727446 * (temp - 60.0).powf(-0.1332047592)).clamp(0.0, 255.0);
        let g = (288.1221695283 * (temp - 60.0).powf(-0.0755148492)).clamp(0.0, 255.0);
        let b = 255.0;
        (r, g, b)
    };
    [(r / 255.0) as f32, (g / 255.0) as f32, (b / 255.0) as f32]
}

/// 谜题目录尚无 skybox.png 时写入默认图
fn ensure_default_skybox_for_puzzle(puzzle: &str) {
    let path = SaveSlot::puzzle(puzzle).storage_path();
    if persistent_storage::read_save_bytes(&path, SKYBOX_FILE).is_some() {
        return;
    }
    if !persistent_storage::write_save_bytes(&path, SKYBOX_FILE, DEFAULT_SKYBOX_PNG) {
        warn!("failed to write default {SKYBOX_FILE} for puzzle `{puzzle}`");
    }
}

fn read_save(slot: &SaveSlot) -> Option<SaveFile> {
    let path = slot.storage_path();
    let meta_text = persistent_storage::read_save_text(&path, META_FILE)?;
    let meta = serde_json::from_str::<SaveMeta>(&meta_text).ok()?;
    let blocks_bytes = persistent_storage::read_save_bytes(&path, BLOCKS_FILE)?;
    let blocks = save_format::decode_blocks(&blocks_bytes).ok()?;
    Some(SaveFile { meta, blocks })
}

fn capture_puzzle_layer(world: &WorldBlocks, hotbar: &SavedHotbar) -> PuzzleLayer {
    let scene_blocks: Vec<SavedBlock> = world
        .blocks
        .iter()
        .filter_map(|(pos, data)| {
            (data.kind.persistent_layer() == Some(PersistentLayer::Puzzle))
                .then_some(saved_block(*pos, *data, None))
        })
        .collect();
    let system_blocks: Vec<SavedBlock> = world
        .system_blocks
        .iter()
        .filter_map(|(pos, data)| {
            (data.kind.persistent_layer() == Some(PersistentLayer::Puzzle)).then_some({
                let settings = world.block_settings.get(pos).cloned();
                saved_block(*pos, *data, settings)
            })
        })
        .collect();

    PuzzleLayer {
        scene_blocks,
        system_blocks,
        hotbar: Some(*hotbar),
    }
}

fn capture_factory_blocks(world: &WorldBlocks) -> Vec<SavedBlock> {
    world
        .blocks
        .iter()
        .filter_map(|(pos, data)| {
            (data.kind.persistent_layer() == Some(PersistentLayer::SolutionFactory)).then_some(
                saved_block(*pos, *data, world.block_settings.get(pos).cloned()),
            )
        })
        .collect()
}

fn capture_wire_face_panels(world: &WorldBlocks) -> Vec<save_format::SavedWireFacePanel> {
    let id_to_pos: std::collections::HashMap<_, _> = world
        .blocks
        .iter()
        .map(|(pos, block)| (block.id, *pos))
        .collect();
    world
        .wire_face_panels
        .iter()
        .filter_map(|face| {
            let pos = id_to_pos.get(&face.block)?;
            Some(save_format::SavedWireFacePanel {
                x: pos.x,
                y: pos.y,
                z: pos.z,
                nx: face.normal.x,
                ny: face.normal.y,
                nz: face.normal.z,
            })
        })
        .collect()
}

fn apply_layer(world: &mut WorldBlocks, layer: PuzzleLayer) {
    for saved in layer.scene_blocks {
        world.insert(saved.pos(), saved.to_block_data());
    }
    for saved in layer.system_blocks {
        world.insert(saved.pos(), saved.to_block_data());
        if let Some(settings) = &saved.settings {
            world.set_block_settings(saved.pos(), settings.clone());
        }
    }
    world.resync_acceptor_structures();
}

fn apply_factory_blocks(world: &mut WorldBlocks, factory_blocks: Vec<SavedBlock>) {
    for saved in factory_blocks {
        if saved.kind.persistent_layer() == Some(PersistentLayer::SolutionFactory) {
            world.insert(saved.pos(), saved.to_block_data());
            if let Some(settings) = &saved.settings {
                world.set_block_settings(saved.pos(), settings.clone());
            }
        }
    }
    world.rebuild_factory_attachments();
}

fn apply_wire_face_panels(world: &mut WorldBlocks, panels: Vec<save_format::SavedWireFacePanel>) {
    for panel in panels {
        let pos = panel.pos();
        let Some(block) = world.blocks.get(&pos).copied() else {
            continue;
        };
        if block.kind != BlockKind::Wire {
            continue;
        }
        world.set_wire_face_panel(MaterialFace::new(block.id, panel.normal()), true);
    }
}

fn saved_block(pos: IVec3, data: BlockData, settings: Option<BlockSettings>) -> SavedBlock {
    let mut saved = SavedBlock::from_block_data(pos, data);
    saved.settings = settings;
    saved
}

pub fn list_saves() -> Vec<String> {
    persistent_storage::list_saves()
}

pub fn list_save_entries() -> Vec<SaveEntry> {
    let mut entries = Vec::new();
    for puzzle in persistent_storage::list_puzzles() {
        let slot = SaveSlot::puzzle(&puzzle);
        entries.push(SaveEntry {
            name: entry_display_name(&slot, &puzzle),
            slot,
            kind: SaveKind::Puzzle,
        });
        for solution in persistent_storage::list_solution_names(&puzzle) {
            let slot = SaveSlot::solution(&puzzle, &solution);
            entries.push(SaveEntry {
                name: entry_display_name(&slot, &solution),
                slot,
                kind: SaveKind::Solution,
            });
        }
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name).then(a.slot.puzzle.cmp(&b.slot.puzzle)));
    entries
}

/// 读 meta.name；没有则用文件夹名
fn entry_display_name(slot: &SaveSlot, fallback: &str) -> String {
    read_save_name(slot).unwrap_or_else(|| fallback.to_string())
}

fn read_save_name(slot: &SaveSlot) -> Option<String> {
    read_save(slot)
        .and_then(|save| save.meta.name)
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
}

pub fn puzzle_names(entries: &[SaveEntry]) -> Vec<String> {
    entries
        .iter()
        .filter(|entry| entry.kind == SaveKind::Puzzle)
        .map(|entry| entry.slot.puzzle.clone())
        .collect()
}

pub fn solution_names_for_puzzle(entries: &[SaveEntry], puzzle: &str) -> Vec<String> {
    entries
        .iter()
        .filter(|entry| entry.kind == SaveKind::Solution && entry.slot.puzzle == puzzle)
        .filter_map(|entry| entry.slot.solution.clone())
        .collect()
}

pub fn next_world_name(existing: &[String]) -> String {
    for index in 1.. {
        let candidate = format!("world_{index}");
        if !existing.iter().any(|name| name == &candidate) {
            return candidate;
        }
    }
    unreachable!()
}

pub fn next_named_save(existing: &[String], base: &str) -> String {
    let base = normalized_save_name(base);
    if base.is_empty() {
        return next_world_name(existing);
    }
    if !existing.iter().any(|name| name == &base) {
        return base;
    }
    for index in 2.. {
        let candidate = format!("{base}_{index}");
        if !existing.iter().any(|name| name == &candidate) {
            return candidate;
        }
    }
    unreachable!()
}

fn sanitize_save_name(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

pub fn normalized_save_name(name: &str) -> String {
    sanitize_save_name(name.trim())
        .trim_matches('_')
        .to_string()
}
