impl StructureState {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn clear_turn_marks(&mut self) {
        self.held_blocks.clear();
        self.moving_structures.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.structure_by_pos.is_empty()
    }

    fn alloc_id(&mut self) -> StructureId {
        self.next_structure_id += 1;
        StructureId(self.next_structure_id)
    }

    fn alloc_head_id(&mut self) -> BlockId {
        self.next_head_id = self.next_head_id.max(1);
        let id = BlockId(self.next_head_id);
        self.next_head_id += 1;
        id
    }

    /// 逻辑头 ID 不得与世界方块 ID 冲突
    fn sync_head_counter(&mut self, world: &WorldBlocks) {
        self.next_head_id = self.next_head_id.max(world.next_block_id).max(1);
    }

    /// 全量重建：工厂连通 + 贴场景 activity + deform + 验收口 + 材料
    pub fn rebuild_for_simulation(&mut self, world: &WorldBlocks) {
        self.rebuild_for_simulation_mode(world, true);
    }

    /// 游戏运行时重建结构，推杆 deform 延迟到结构首次实际动作
    pub fn rebuild_for_runtime(&mut self, world: &WorldBlocks) {
        self.rebuild_for_simulation_mode(world, false);
    }

    /// 按需重建完整结构；模拟开局可延迟尚未动作的推杆 deform
    fn rebuild_for_simulation_mode(&mut self, world: &WorldBlocks, rebuild_deform: bool) {
        let next_head = self.next_head_id.max(world.next_block_id).max(1);
        *self = Self::default();
        self.next_head_id = next_head;
        self.append_factory_structures(world);
        self.refresh_all_factory_activity_from_scene(world);
        if rebuild_deform {
            self.rebuild_all_factory_deform(world);
        }
        self.append_acceptor_structures(world);
        self.append_material_structures(world, &HashMap::new(), &HashMap::new());
    }

    /// 开局轻量：已有工厂缓存则只刷新材料/验收口；否则全量重建
    pub fn refresh_for_simulation_start(&mut self, world: &WorldBlocks) {
        self.clear_turn_marks();
        if !self
            .structures
            .values()
            .any(|structure| structure.kind == StructureKind::Factory)
        {
            self.rebuild_for_simulation_mode(world, false);
            return;
        }
        self.acceptor_structures.clear();
        self.append_acceptor_structures(world);
        self.refresh_material_structures(world);
    }

    pub fn acceptor_structures(&self) -> &[AcceptorStructure] {
        &self.acceptor_structures
    }

    pub fn increment_acceptor_count(&mut self, index: usize) {
        if let Some(structure) = self.acceptor_structures.get_mut(index) {
            structure.count = structure.count.saturating_add(1);
        }
    }

    /// 调试：仅重建工厂连通与 deform
    pub fn rebuild_factory_for_debug(&mut self, world: &WorldBlocks) {
        self.retain_factory_only();
        self.append_factory_structures(world);
        self.refresh_all_factory_activity_from_scene(world);
        self.rebuild_all_factory_deform(world);
    }

    pub fn refresh_material_structures(&mut self, world: &WorldBlocks) {
        let mut previous_ids: HashMap<Vec<u64>, StructureId> = HashMap::new();
        let mut previous_support: HashMap<StructureId, Vec<GravitySupportContact>> = HashMap::new();
        for (id, structure) in &self.structures {
            if structure.kind != StructureKind::Material {
                continue;
            }
            let mut members: Vec<u64> = structure
                .positions
                .iter()
                .filter_map(|pos| world.blocks.get(pos).map(|block| block.id.0))
                .collect();
            members.sort_unstable();
            previous_ids.insert(members, *id);
            previous_support.insert(*id, structure.gravity_support.clone());
        }
        self.retain_factory_only();
        self.append_material_structures(world, &previous_ids, &previous_support);
    }

    /// 编辑变更：只拆/重建触及的材料连通，避免整图材料洪水
    fn apply_material_edit(&mut self, world: &WorldBlocks, changed: &HashSet<IVec3>) {
        let mut stale_ids: HashSet<StructureId> = HashSet::new();
        let mut seeds: HashSet<IVec3> = HashSet::new();
        for &pos in changed {
            if let Some(&id) = self.structure_by_pos.get(&pos) {
                if self
                    .structures
                    .get(&id)
                    .is_some_and(|structure| structure.kind == StructureKind::Material)
                {
                    stale_ids.insert(id);
                }
            }
            if world.is_material_at(pos) {
                seeds.insert(pos);
            }
        }

        let id_to_pos = material_id_to_pos(world);
        let weld_neighbors = material_weld_neighbors(world);
        for &pos in changed {
            let Some(block) = world
                .blocks
                .get(&pos)
                .filter(|block| block.kind.is_material() && !block.id.is_none())
            else {
                continue;
            };
            for &other_id in weld_neighbors.get(&block.id).into_iter().flatten() {
                let Some(&other_pos) = id_to_pos.get(&other_id) else {
                    continue;
                };
                seeds.insert(other_pos);
                if let Some(&id) = self.structure_by_pos.get(&other_pos) {
                    if self
                        .structures
                        .get(&id)
                        .is_some_and(|structure| structure.kind == StructureKind::Material)
                    {
                        stale_ids.insert(id);
                    }
                }
            }
        }

        let mut previous_ids: HashMap<Vec<u64>, StructureId> = HashMap::new();
        let mut previous_support: HashMap<StructureId, Vec<GravitySupportContact>> = HashMap::new();
        for id in stale_ids {
            let Some(structure) = self.structures.remove(&id) else {
                continue;
            };
            let mut members: Vec<u64> = structure
                .positions
                .iter()
                .filter_map(|pos| world.blocks.get(pos).map(|block| block.id.0))
                .collect();
            members.sort_unstable();
            if !members.is_empty() {
                previous_ids.insert(members, id);
            }
            previous_support.insert(id, structure.gravity_support.clone());
            for pos in &structure.positions {
                self.structure_by_pos.remove(pos);
                if world.is_material_at(*pos) {
                    seeds.insert(*pos);
                }
            }
        }

        if seeds.is_empty() {
            return;
        }
        let mut starts: Vec<IVec3> = seeds.into_iter().collect();
        starts.sort_by_key(|pos| (pos.x, pos.y, pos.z));
        self.append_material_at_starts(
            world,
            &starts,
            &previous_ids,
            &previous_support,
            &id_to_pos,
            &weld_neighbors,
        );
    }

    fn retain_factory_only(&mut self) {
        self.structures
            .retain(|_, structure| structure.kind == StructureKind::Factory);
        self.structure_by_pos.clear();
        for (id, structure) in &self.structures {
            for pos in &structure.positions {
                self.structure_by_pos.insert(*pos, *id);
            }
        }
    }

    fn append_factory_structures(&mut self, world: &WorldBlocks) {
        let starts: Vec<IVec3> = world
            .blocks
            .iter()
            .filter_map(|(pos, block)| block.kind.is_factory().then_some(*pos))
            .collect();
        self.append_connected_factory_structures(world, starts);
    }

    fn append_acceptor_structures(&mut self, world: &WorldBlocks) {
        for stored in &world.acceptor_structures {
            self.acceptor_structures.push(AcceptorStructure {
                id: stored.id,
                positions: stored.positions.iter().copied().collect(),
                count: 0,
            });
        }
    }

    fn append_material_structures(
        &mut self,
        world: &WorldBlocks,
        previous_ids: &HashMap<Vec<u64>, StructureId>,
        previous_support: &HashMap<StructureId, Vec<GravitySupportContact>>,
    ) {
        let id_to_pos = material_id_to_pos(world);
        let weld_neighbors = material_weld_neighbors(world);
        let mut starts: Vec<IVec3> = world
            .blocks
            .iter()
            .filter_map(|(pos, block)| block.kind.is_material().then_some(*pos))
            .collect();
        starts.sort_by_key(|pos| (pos.x, pos.y, pos.z));
        self.append_material_at_starts(
            world,
            &starts,
            previous_ids,
            previous_support,
            &id_to_pos,
            &weld_neighbors,
        );
    }

    fn append_material_at_starts(
        &mut self,
        world: &WorldBlocks,
        starts: &[IVec3],
        previous_ids: &HashMap<Vec<u64>, StructureId>,
        previous_support: &HashMap<StructureId, Vec<GravitySupportContact>>,
        id_to_pos: &HashMap<BlockId, IVec3>,
        weld_neighbors: &HashMap<BlockId, Vec<BlockId>>,
    ) {
        let mut handled = self
            .structure_by_pos
            .keys()
            .copied()
            .collect::<HashSet<_>>();

        for &start in starts {
            if handled.contains(&start) || !world.is_material_at(start) {
                continue;
            }
            let positions = material_structure_from(world, start, id_to_pos, weld_neighbors);
            let mut members: Vec<u64> = positions
                .iter()
                .filter_map(|pos| world.blocks.get(pos).map(|block| block.id.0))
                .collect();
            members.sort_unstable();
            let id = previous_ids
                .get(&members)
                .copied()
                .unwrap_or_else(|| self.alloc_id());
            let gravity_support = previous_support.get(&id).cloned().unwrap_or_default();
            for pos in &positions {
                handled.insert(*pos);
                self.structure_by_pos.insert(*pos, id);
            }
            self.structures.insert(
                id,
                Structure {
                    id,
                    kind: StructureKind::Material,
                    positions,
                    activity: FactoryActivity::Active,
                    freedom: StructureFreedom::All,
                    gravity_support,
                    head_of: HashMap::new(),
                    body_of_head: HashMap::new(),
                    deform_groups: Vec::new(),
                    action_to_groups: HashMap::new(),
                    scene_touching: HashSet::new(),
                },
            );
        }
    }

    fn append_connected_factory_structures(
        &mut self,
        world: &WorldBlocks,
        starts: impl IntoIterator<Item = IVec3>,
    ) {
        let mut handled: HashSet<IVec3> = self.structure_by_pos.keys().copied().collect();
        let mut starts: Vec<IVec3> = starts
            .into_iter()
            .filter(|pos| world.is_factory_at(*pos) && !handled.contains(pos))
            .collect();
        starts.sort_by_key(|pos| (pos.x, pos.y, pos.z));

        for start in starts {
            if handled.contains(&start) || !world.is_factory_at(start) {
                continue;
            }

            let positions = factory_structure(world, start);
            let id = self.alloc_id();
            for pos in &positions {
                handled.insert(*pos);
                self.structure_by_pos.insert(*pos, id);
            }
            self.structures.insert(
                id,
                Structure {
                    id,
                    kind: StructureKind::Factory,
                    positions,
                    activity: FactoryActivity::Active,
                    freedom: StructureFreedom::All,
                    gravity_support: Vec::new(),
                    head_of: HashMap::new(),
                    body_of_head: HashMap::new(),
                    deform_groups: Vec::new(),
                    action_to_groups: HashMap::new(),
                    scene_touching: HashSet::new(),
                },
            );
        }
    }

    /// 贴场景 → Inactive + Freedom::None；否则 Active + All。不做结构间传播。
    fn refresh_factory_activity_from_scene(
        &mut self,
        world: &WorldBlocks,
        ids: impl IntoIterator<Item = StructureId>,
    ) {
        for id in ids {
            let Some(structure) = self.structures.get_mut(&id) else {
                continue;
            };
            if structure.kind != StructureKind::Factory {
                continue;
            }
            structure.scene_touching = structure
                .positions
                .iter()
                .copied()
                .filter(|pos| {
                    signal_offsets().into_iter().any(|offset| {
                        let neighbor = *pos + offset;
                        world.is_scene_at(neighbor)
                            && !is_blocked_factory_connection(world, *pos, neighbor)
                    })
                })
                .collect();
            if !structure.scene_touching.is_empty() {
                structure.activity = FactoryActivity::Inactive;
                structure.freedom = StructureFreedom::None;
            } else {
                structure.activity = FactoryActivity::Active;
                structure.freedom = StructureFreedom::All;
            }
        }
    }

    fn refresh_all_factory_activity_from_scene(&mut self, world: &WorldBlocks) {
        let factory_ids: Vec<StructureId> = self
            .structures
            .iter()
            .filter(|(_, structure)| structure.kind == StructureKind::Factory)
            .map(|(id, _)| *id)
            .collect();
        self.refresh_factory_activity_from_scene(world, factory_ids);
    }

    /// 编辑变更：局部合并/拆分工厂连通并重算受影响 deform
    pub fn apply_factory_edit(&mut self, world: &WorldBlocks, changed: &HashSet<IVec3>) {
        if changed.is_empty() {
            return;
        }

        let mut seed_positions: HashSet<IVec3> = HashSet::new();
        let mut stale_ids: HashSet<StructureId> = HashSet::new();

        for &pos in changed {
            if let Some(id) = self.structure_by_pos.get(&pos).copied() {
                if let Some(structure) = self.structures.get(&id) {
                    if structure.kind == StructureKind::Factory {
                        stale_ids.insert(id);
                        seed_positions.extend(structure.positions.iter().copied());
                    }
                }
            }
            for offset in signal_offsets() {
                let neighbor = pos + offset;
                if let Some(id) = self.structure_by_pos.get(&neighbor).copied() {
                    if let Some(structure) = self.structures.get(&id) {
                        if structure.kind == StructureKind::Factory {
                            stale_ids.insert(id);
                            seed_positions.extend(structure.positions.iter().copied());
                        }
                    }
                }
            }
            if world.is_factory_at(pos) {
                seed_positions.insert(pos);
            }
        }

        for id in &stale_ids {
            if let Some(structure) = self.structures.remove(id) {
                for pos in &structure.positions {
                    self.structure_by_pos.remove(pos);
                }
            }
        }

        let mut rebuild_seeds: Vec<IVec3> = seed_positions
            .into_iter()
            .filter(|pos| world.is_factory_at(*pos) && !self.structure_by_pos.contains_key(pos))
            .collect();
        rebuild_seeds.sort_by_key(|pos| (pos.x, pos.y, pos.z));

        let before_ids: HashSet<StructureId> = self.structures.keys().copied().collect();
        self.append_connected_factory_structures(world, rebuild_seeds);
        let new_ids: Vec<StructureId> = self
            .structures
            .keys()
            .copied()
            .filter(|id| !before_ids.contains(id))
            .collect();

        if !new_ids.is_empty() {
            self.refresh_factory_activity_from_scene(world, new_ids.iter().copied());
        }
        self.apply_material_edit(world, changed);
    }

    fn rebuild_all_factory_deform(&mut self, world: &WorldBlocks) {
        let ids: Vec<StructureId> = self
            .structures
            .iter()
            .filter(|(_, s)| s.kind == StructureKind::Factory)
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            self.rebuild_deform_for(world, id);
        }
    }
}
