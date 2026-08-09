impl SaveFile {
    fn puzzle(layer: PuzzleLayer, player: Option<PlayerSave>) -> Self {
        Self {
            meta: SaveMeta {
                version: SAVE_VERSION,
                kind: SaveMetaKind::Puzzle,
                name: None,
                created_at: None,
                updated_at: None,
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

    fn free(world: FreeWorldCapture, player: Option<PlayerSave>) -> Self {
        Self {
            meta: SaveMeta {
                version: SAVE_VERSION,
                kind: SaveMetaKind::Free,
                name: None,
                created_at: None,
                updated_at: None,
                puzzle_id: None,
                hotbar: world.hotbar,
                player,
                sun: None,
                ambient: None,
                sun_direction: None,
            },
            blocks: SaveBlocksData {
                scene_blocks: world.scene_blocks,
                system_blocks: world.system_blocks,
                factory_blocks: world.factory_blocks,
                wire_face_panels: world.wire_face_panels,
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
                created_at: None,
                updated_at: None,
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
            SaveMetaKind::Free => SaveKind::Free,
        }
    }

    fn into_loaded(self, slot: &SaveSlot) -> Option<LoadedSave> {
        match self.meta.kind {
            SaveMetaKind::Solution => {
                if slot.kind() != SaveKind::Solution || slot.solution.is_none() {
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
                if slot.kind() != SaveKind::Puzzle || slot.solution.is_some() {
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
            SaveMetaKind::Free => {
                if slot.kind() != SaveKind::Free || slot.solution.is_some() {
                    return None;
                }
                let lighting = resolve_lighting(&self.meta);
                let hotbar = self.meta.hotbar;
                let mut world = WorldBlocks::default();
                apply_layer(
                    &mut world,
                    PuzzleLayer {
                        scene_blocks: self.blocks.scene_blocks,
                        system_blocks: self.blocks.system_blocks,
                        hotbar: None,
                    },
                );
                apply_factory_blocks(&mut world, self.blocks.factory_blocks);
                apply_wire_face_panels(&mut world, self.blocks.wire_face_panels);
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
    let now = unix_now_secs();
    // 覆盖保存：名字 / 光照 / 创建时间只保留磁盘上已有的；每次写入刷新 updated_at
    if let Some(existing) = read_save(slot) {
        save.meta.name = existing.meta.name;
        save.meta.sun = existing.meta.sun;
        save.meta.ambient = existing.meta.ambient;
        save.meta.sun_direction = existing.meta.sun_direction;
        save.meta.created_at = existing.meta.created_at.or(Some(now));
    } else {
        save.meta.created_at = Some(now);
    }
    save.meta.updated_at = Some(now);
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

fn unix_now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
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
