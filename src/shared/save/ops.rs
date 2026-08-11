pub fn save_puzzle(
    world: &WorldBlocks,
    slot: &SaveSlot,
    hotbar: &SavedHotbar,
    player: Option<PlayerSave>,
) -> bool {
    if slot.kind() != SaveKind::Puzzle || slot.solution.is_some() {
        return false;
    }
    write_save(
        slot,
        SaveFile::puzzle(capture_puzzle_layer(world, hotbar), player),
    )
}

/// 保存 Free 世界（scene + system + factory + panels；不含材料）
pub fn save_free(
    world: &WorldBlocks,
    slot: &SaveSlot,
    hotbar: &SavedHotbar,
    player: Option<PlayerSave>,
) -> bool {
    if slot.kind() != SaveKind::Free || slot.solution.is_some() {
        return false;
    }
    write_save(
        slot,
        SaveFile::free(capture_free_world(world, hotbar), player),
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
    create_top_level_from_template(name, SaveKind::Puzzle)
}

/// 从默认模板新建 Free 世界；成功返回槽位（创建时写入名字）
pub fn create_free_from_default_template(name: &str) -> Option<SaveSlot> {
    create_top_level_from_template(name, SaveKind::Free)
}

/// 创建仅含原点草地方块的 Free 测试存档
pub fn create_test_free_from_single_block(name: &str) -> Option<SaveSlot> {
    let slot = allocate_free_slot(name)?;
    let grass = oif_sim::blocks::resolve_scene_id("grass");
    let mut save = SaveFile::free(
        FreeWorldCapture {
            scene_blocks: vec![SavedBlock {
                x: 0,
                y: 0,
                z: 0,
                kind: BlockKind::Scene(grass),
                facing: None,
                settings: None,
            }],
            system_blocks: Vec::new(),
            factory_blocks: Vec::new(),
            wire_face_panels: Vec::new(),
            hotbar: None,
        },
        Some(PlayerSave {
            x: 0.5,
            y: 2.78,
            z: 10.5,
            yaw: 0.0,
            pitch: -0.15,
            flying: false,
        }),
    );
    let now = unix_now_secs();
    save.meta.name = Some(name.trim().to_string());
    save.meta.created_at = Some(now);
    save.meta.updated_at = Some(now);
    let meta = serde_json::to_string_pretty(&save.meta).ok()?;
    let blocks = save_format::encode_blocks(&save.blocks);
    let path = slot.storage_path();
    persistent_storage::write_ephemeral_save(&path, META_FILE, meta.as_bytes());
    persistent_storage::write_ephemeral_save(&path, BLOCKS_FILE, &blocks);
    Some(slot)
}

/// 用默认模板写顶层 Puzzle/Free 存档
fn create_top_level_from_template(name: &str, kind: SaveKind) -> Option<SaveSlot> {
    let slot = match kind {
        SaveKind::Puzzle => allocate_puzzle_slot(name)?,
        SaveKind::Free => allocate_free_slot(name)?,
        SaveKind::Solution => return None,
    };
    let path = slot.storage_path();
    let meta_text = load_template_text(META_FILE).unwrap_or_else(|| TEMPLATE_META_JSON.to_string());
    let meta = match serde_json::from_str::<SaveMeta>(&meta_text) {
        Ok(mut meta) => {
            meta.kind = match kind {
                SaveKind::Puzzle => SaveMetaKind::Puzzle,
                SaveKind::Free => SaveMetaKind::Free,
                SaveKind::Solution => return None,
            };
            meta.name = Some(name.trim().to_string());
            let now = unix_now_secs();
            meta.created_at = Some(now);
            meta.updated_at = Some(now);
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
            SaveKind::Free => SaveSlot::free(&sanitized),
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
    allocate_top_level_slot(name, SaveKind::Puzzle)
}

fn allocate_free_slot(name: &str) -> Option<SaveSlot> {
    allocate_top_level_slot(name, SaveKind::Free)
}

/// 在 Puzzle/Free 共享命名空间中分配顶层槽位
fn allocate_top_level_slot(name: &str, kind: SaveKind) -> Option<SaveSlot> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let storage = next_named_save(&persistent_storage::list_top_level_names(), name);
    if storage.is_empty() {
        return None;
    }
    let slot = match kind {
        SaveKind::Puzzle => SaveSlot::puzzle(&storage),
        SaveKind::Free => SaveSlot::free(&storage),
        SaveKind::Solution => return None,
    };
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

/// 暂停菜单“存档设置”使用的可编辑配置快照。
#[derive(Clone, Debug)]
pub struct SaveSettingsData {
    pub light_position: Option<Vec3>,
    pub light_direction: Option<Vec3>,
    pub light_intensity: f32,
    pub solution_spawn: Option<PlayerSave>,
    pub factory_block_filter: FactoryBlockFilter,
}

impl Default for SaveSettingsData {
    fn default() -> Self {
        let lighting = PuzzleLighting::default();
        Self {
            light_position: lighting.position,
            light_direction: lighting.direction,
            light_intensity: lighting.illuminance,
            solution_spawn: None,
            factory_block_filter: FactoryBlockFilter::default(),
        }
    }
}

/// 读取顶层 Free/Puzzle 的存档设置；Solution 设置继承所属 Puzzle。
pub fn read_save_settings(slot: &SaveSlot) -> Option<SaveSettingsData> {
    let save = read_save(slot)?;
    let sun_slot = if save.meta.kind == SaveMetaKind::Solution {
        SaveSlot::puzzle(&slot.puzzle)
    } else {
        slot.clone()
    };
    let lighting = if save.meta.kind == SaveMetaKind::Solution {
        read_puzzle_lighting(&slot.puzzle)
    } else {
        resolve_lighting(&save.meta)
    };
    let settings_save = if save.meta.kind == SaveMetaKind::Solution {
        read_save(&sun_slot)?
    } else {
        save
    };
    Some(SaveSettingsData {
        light_position: lighting.position,
        light_direction: lighting.direction,
        light_intensity: lighting.illuminance,
        solution_spawn: settings_save.meta.solution_spawn,
        factory_block_filter: settings_save
            .meta
            .factory_block_filter
            .unwrap_or_default(),
    })
}

/// 只写入存档设置，不改变方块数据和玩家脏状态。
pub fn write_save_settings(slot: &SaveSlot, settings: &SaveSettingsData) -> bool {
    let Some(mut save) = read_save(slot) else {
        return false;
    };
    let mut sun = save.meta.sun.take().unwrap_or_default();
    sun.position = settings.light_position.map(|value| value.to_array());
    sun.direction = settings.light_direction.map(|value| value.to_array());
    sun.illuminance = Some(settings.light_intensity.max(0.0));
    save.meta.sun = Some(sun);
    if save.meta.kind == SaveMetaKind::Puzzle {
        save.meta.solution_spawn = settings.solution_spawn.clone();
        save.meta.factory_block_filter = Some(settings.factory_block_filter.clone());
    }
    save.meta.updated_at = Some(unix_now_secs());
    let Ok(meta) = serde_json::to_string_pretty(&save.meta) else {
        return false;
    };
    persistent_storage::write_save_text(&slot.storage_path(), META_FILE, &meta)
}

/// 替换存档天空盒图片；字节由渲染层按水平十字格式解码。
pub fn write_save_skybox(slot: &SaveSlot, bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    persistent_storage::write_save_bytes(&slot.storage_path(), SKYBOX_FILE, bytes)
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
