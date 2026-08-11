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

/// 存档寻址：Puzzle/Free 为顶层目录，Solution 在 Puzzle 的 `solutions/` 子目录下
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SaveSlot {
    pub puzzle: String,
    pub solution: Option<String>,
    kind: SaveKind,
}

impl SaveSlot {
    pub fn puzzle(name: impl Into<String>) -> Self {
        Self {
            puzzle: normalized_save_name(&name.into()),
            solution: None,
            kind: SaveKind::Puzzle,
        }
    }

    /// Free 与 Puzzle 同为顶层目录，靠 meta.kind 区分
    pub fn free(name: impl Into<String>) -> Self {
        Self {
            puzzle: normalized_save_name(&name.into()),
            solution: None,
            kind: SaveKind::Free,
        }
    }

    pub fn solution(puzzle: impl Into<String>, solution: impl Into<String>) -> Self {
        Self {
            puzzle: normalized_save_name(&puzzle.into()),
            solution: Some(normalized_save_name(&solution.into())),
            kind: SaveKind::Solution,
        }
    }

    pub fn kind(&self) -> SaveKind {
        self.kind
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
            [name] if !name.is_empty() => {
                // 单段路径：读 meta.kind 区分 Puzzle / Free；无 meta 默认 Puzzle
                match persistent_storage::save_meta_kind(name) {
                    Some("free") => Some(Self::free(*name)),
                    _ => Some(Self::puzzle(*name)),
                }
            }
            [puzzle, "solutions", solution] if !puzzle.is_empty() && !solution.is_empty() => {
                Some(Self::solution(*puzzle, *solution))
            }
            _ => None,
        }
    }
}

/// 读取存档封面 PNG（vault 优先；桌面再回退磁盘截图路径）
pub fn read_cover_png(slot: &SaveSlot) -> Option<Vec<u8>> {
    let path = slot.storage_path();
    if let Some(bytes) = persistent_storage::read_save_bytes(&path, COVER_FILE) {
        if !bytes.is_empty() {
            return Some(bytes);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let disk = crate::shared::platform::saves_directory()
            .join(&path)
            .join(COVER_FILE);
        return std::fs::read(disk).ok().filter(|bytes| !bytes.is_empty());
    }
    #[cfg(target_arch = "wasm32")]
    {
        None
    }
}

#[derive(Resource, Default)]
pub struct SaveState {
    pub current: Option<SaveSlot>,
    pub current_kind: Option<SaveKind>,
    pub entries: Vec<SaveEntry>,
    pub selected_puzzle: Option<String>,
    /// 存档列表中选中的方案（storage 名）
    pub selected_solution: Option<String>,
    selected_puzzle_solutions: Vec<SaveEntry>,
}

impl SaveState {
    pub fn refresh(&mut self) {
        self.entries = list_save_entries();
        self.refresh_selected_puzzle_solutions();
        self.prune_selected_solution();
    }

    pub fn puzzles(&self) -> Vec<&SaveEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.kind == SaveKind::Puzzle)
            .collect()
    }

    /// 左侧列表用的顶层世界（Puzzle + Free）
    pub fn top_level_worlds(&self) -> Vec<&SaveEntry> {
        self.entries
            .iter()
            .filter(|entry| matches!(entry.kind, SaveKind::Puzzle | SaveKind::Free))
            .collect()
    }

    pub fn select_puzzle(&mut self, puzzle: Option<String>) {
        if self.selected_puzzle == puzzle {
            return;
        }
        self.selected_puzzle = puzzle;
        self.selected_solution = None;
        self.refresh_selected_puzzle_solutions();
    }

    pub fn select_solution(&mut self, solution: Option<String>) {
        if self.selected_solution == solution {
            return;
        }
        self.selected_solution = solution;
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
        self.prune_selected_solution();
    }

    fn prune_selected_solution(&mut self) {
        let Some(selected) = self.selected_solution.as_deref() else {
            return;
        };
        let still_there = self
            .selected_puzzle_solutions
            .iter()
            .any(|entry| entry.slot.solution.as_deref() == Some(selected));
        if !still_there {
            self.selected_solution = None;
        }
    }
}

#[derive(Clone)]
pub struct SaveEntry {
    pub slot: SaveSlot,
    pub name: String,
    pub kind: SaveKind,
    /// 创建时间（unix 秒）；旧档可能为 None
    pub created_at: Option<u64>,
    /// 上次保存时间（unix 秒）；旧档可能为 None
    pub updated_at: Option<u64>,
    /// 玩家最近进入世界的时间（unix 秒）；旧档可能为 None
    pub last_play_time: Option<u64>,
    /// 是否收藏
    pub favorite: bool,
}

impl SaveEntry {
    pub fn puzzle_id(&self) -> Option<&str> {
        (self.kind == SaveKind::Solution).then_some(self.slot.puzzle.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum SaveKind {
    Puzzle,
    Free,
    Solution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SaveMetaKind {
    Puzzle,
    Free,
    Solution,
}

/// Solution 背包中的工厂方块过滤模式。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FactoryBlockFilterMode {
    #[default]
    Blacklist,
    Whitelist,
}

/// Puzzle 为其 Solution 提供的工厂方块黑白名单。
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct FactoryBlockFilter {
    #[serde(default)]
    pub mode: FactoryBlockFilterMode,
    #[serde(default)]
    pub kinds: Vec<BlockKind>,
}

impl FactoryBlockFilter {
    pub fn allows(&self, kind: BlockKind) -> bool {
        if !kind.is_factory() {
            return true;
        }
        let listed = self.kinds.contains(&kind);
        match self.mode {
            FactoryBlockFilterMode::Blacklist => !listed,
            FactoryBlockFilterMode::Whitelist => listed,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SaveMeta {
    version: u32,
    kind: SaveMetaKind,
    /// 存档显示名，可含中文。省略时用文件夹名。文件夹本身仍是 sanitize 后的 id（仅 ASCII 字母数字/_/-）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// 首次创建时间（unix 秒）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    created_at: Option<u64>,
    /// 上次写入时间（unix 秒）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    updated_at: Option<u64>,
    /// 玩家最近进入世界的时间（unix 秒）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_play_time: Option<u64>,
    /// 是否收藏
    #[serde(default)]
    favorite: bool,
    #[serde(default)]
    puzzle_id: Option<String>,
    #[serde(default)]
    hotbar: Option<SavedHotbar>,
    #[serde(default)]
    player: Option<PlayerSave>,
    /// 编辑 Puzzle 时配置的 Solution 玩家出生点。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    solution_spawn: Option<PlayerSave>,
    /// 编辑 Puzzle 时配置的 Solution 工厂方块过滤规则。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    factory_block_filter: Option<FactoryBlockFilter>,
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
    /// 平行光实体位置；方向仍由 `direction` 表示。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    position: Option<[f32; 3]>,
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
    pub position: Option<Vec3>,
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
            position: None,
            direction: None,
            illuminance: 9500.0,
            color: Color::srgb(1.0, 0.97, 0.92),
            skybox_brightness: 500.0,
            ambient_color: Color::srgb(0.90, 0.94, 1.0),
            ambient_brightness: 340.0,
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

#[derive(Clone)]
pub struct LoadedSave {
    pub world: WorldBlocks,
    pub puzzle_snapshot: Option<WorldBlocks>,
    pub puzzle_id: Option<String>,
    pub hotbar: Option<SavedHotbar>,
    pub player: Option<PlayerSave>,
    pub solution_spawn: Option<PlayerSave>,
    pub factory_block_filter: Option<FactoryBlockFilter>,
    /// 谜题光照（来自 puzzle meta；solution 读所属 puzzle）
    pub lighting: PuzzleLighting,
}
