fn capture_puzzle_layer(world: &WorldBlocks, hotbar: &SavedHotbar) -> PuzzleLayer {
    let scene_blocks: Vec<SavedBlock> = world
        .blocks()
        .iter()
        .filter_map(|(pos, data)| {
            (data.kind.persistent_layer() == Some(PersistentLayer::Puzzle))
                .then_some(saved_block(*pos, *data, None))
        })
        .collect();
    let system_blocks: Vec<SavedBlock> = world
        .system_blocks()
        .iter()
        .filter_map(|(pos, data)| {
            (data.kind.persistent_layer() == Some(PersistentLayer::Puzzle)).then_some({
                let settings = world.block_settings().get(pos).cloned();
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

/// Free：谜题层 + 工厂块 + 灯面板（不含材料）
struct FreeWorldCapture {
    scene_blocks: Vec<SavedBlock>,
    system_blocks: Vec<SavedBlock>,
    factory_blocks: Vec<SavedBlock>,
    wire_face_panels: Vec<save_format::SavedWireFacePanel>,
    hotbar: Option<SavedHotbar>,
}

fn capture_free_world(world: &WorldBlocks, hotbar: &SavedHotbar) -> FreeWorldCapture {
    let layer = capture_puzzle_layer(world, hotbar);
    FreeWorldCapture {
        scene_blocks: layer.scene_blocks,
        system_blocks: layer.system_blocks,
        factory_blocks: capture_factory_blocks(world),
        wire_face_panels: capture_wire_face_panels(world),
        hotbar: layer.hotbar,
    }
}

fn capture_factory_blocks(world: &WorldBlocks) -> Vec<SavedBlock> {
    world
        .blocks()
        .iter()
        .filter_map(|(pos, data)| {
            (data.kind.persistent_layer() == Some(PersistentLayer::SolutionFactory)).then_some(
                saved_block(*pos, *data, world.block_settings().get(pos).cloned()),
            )
        })
        .collect()
}

fn capture_wire_face_panels(world: &WorldBlocks) -> Vec<save_format::SavedWireFacePanel> {
    let id_to_pos: std::collections::HashMap<_, _> = world
        .blocks()
        .iter()
        .map(|(pos, block)| (block.id, *pos))
        .collect();
    world
        .wire_face_panels()
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
        let Some(block) = world.blocks().get(&pos).copied() else {
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
