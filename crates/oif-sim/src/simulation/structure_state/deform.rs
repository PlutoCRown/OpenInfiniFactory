impl StructureState {
    /// 为单个工厂结构分配逻辑头并构建共轴 DeformGroup
    fn rebuild_deform_for(&mut self, world: &WorldBlocks, id: StructureId) {
        self.sync_head_counter(world);
        let Some(structure) = self.structures.get(&id) else {
            return;
        };
        if structure.kind != StructureKind::Factory {
            return;
        }
        let positions = structure.positions.clone();
        let mut prev_heads = structure.head_of.clone();

        let scene_touching: HashSet<IVec3> = positions
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

        // body → head；复用旧 head id
        let mut head_of: HashMap<BlockId, BlockId> = HashMap::new();
        let mut body_of_head: HashMap<BlockId, BlockId> = HashMap::new();
        let mut body_pos: HashMap<BlockId, IVec3> = HashMap::new();
        let mut facing_of: HashMap<BlockId, IVec3> = HashMap::new();

        for &pos in &positions {
            let Some(block) = world.blocks.get(&pos) else {
                continue;
            };
            let Some(MovementRule::PoweredTranslate { source, .. }) =
                block.kind.movement_rule(block.facing)
            else {
                continue;
            };
            let head_id = prev_heads
                .remove(&block.id)
                .unwrap_or_else(|| self.alloc_head_id());
            head_of.insert(block.id, head_id);
            body_of_head.insert(head_id, block.id);
            body_pos.insert(block.id, pos);
            facing_of.insert(block.id, source);
        }

        // 无向邻接 + 有向 body→head
        let mut undirected: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();
        let link_u = |adj: &mut HashMap<BlockId, HashSet<BlockId>>, a: BlockId, b: BlockId| {
            adj.entry(a).or_default().insert(b);
            adj.entry(b).or_default().insert(a);
        };

        let id_at: HashMap<IVec3, BlockId> = positions
            .iter()
            .filter_map(|pos| world.blocks.get(pos).map(|b| (*pos, b.id)))
            .collect();

        for (&body, &head) in &head_of {
            let Some(&bpos) = body_pos.get(&body) else {
                continue;
            };
            let Some(&forward) = facing_of.get(&body) else {
                continue;
            };
            // 体—头仅有向；头接到正面（贴脸对向则头—头，避免绕回本体）
            let mut front = bpos + forward;
            while world
                .blocks
                .get(&front)
                .is_some_and(|b| b.kind == BlockKind::PusherHead)
            {
                front += forward;
            }
            if let Some(front_block) = world.blocks.get(&front) {
                let face_to_face = matches!(
                    front_block.kind.movement_rule(front_block.facing),
                    Some(MovementRule::PoweredTranslate { source, .. })
                        if bpos_facing_toward(front, source, bpos)
                );
                if face_to_face {
                    if let Some(&other_head) = head_of.get(&front_block.id) {
                        link_u(&mut undirected, head, other_head);
                    }
                } else if let Some(&front_id) = id_at.get(&front) {
                    link_u(&mut undirected, head, front_id);
                }
            }
        }

        for &pos in &positions {
            let Some(&block_id) = id_at.get(&pos) else {
                continue;
            };
            let Some(block) = world.blocks.get(&pos) else {
                continue;
            };
            let forward_opt = matches!(
                block.kind.movement_rule(block.facing),
                Some(MovementRule::PoweredTranslate { .. })
            )
            .then(|| block.facing.forward_ivec3());

            for offset in signal_offsets() {
                if forward_opt == Some(offset) {
                    continue; // 正面只经头连
                }
                let neighbor = pos + offset;
                let Some(&neighbor_id) = id_at.get(&neighbor) else {
                    continue;
                };
                if is_blocked_factory_connection(world, pos, neighbor)
                    || is_blocked_factory_connection(world, neighbor, pos)
                {
                    continue;
                }
                // 邻块若是朝向本格的推杆，已有「邻体→头→本格」，不再加体—体边
                let neighbor_faces_here = world.blocks.get(&neighbor).is_some_and(|b| {
                    matches!(
                        b.kind.movement_rule(b.facing),
                        Some(MovementRule::PoweredTranslate { source, .. })
                            if bpos_facing_toward(neighbor, source, pos)
                    )
                });
                if neighbor_faces_here {
                    continue;
                }
                link_u(&mut undirected, block_id, neighbor_id);
            }
        }

        // 推杆轴向：东西 / 南北 / 上下（同轴才共组；横切枚举也只对同轴同伴）
        let axis_of = |fwd: IVec3| -> u8 {
            if fwd.x != 0 {
                0
            } else if fwd.z != 0 {
                1
            } else {
                2
            }
        };

        // 在给定切断集下传播；碰到 forbidden=成环。有向边仅当其体未切断时可走。
        let propagate_with_cuts = |start: BlockId,
                                   forbidden: BlockId,
                                   cuts: &HashSet<BlockId>,
                                   undirected_only: bool|
         -> Option<Vec<BlockId>> {
            let mut seen = HashSet::from([start]);
            let mut queue = VecDeque::from([start]);
            while let Some(node) = queue.pop_front() {
                if let Some(neighbors) = undirected.get(&node) {
                    for &next in neighbors {
                        if next == forbidden {
                            return None;
                        }
                        if seen.insert(next) {
                            queue.push_back(next);
                        }
                    }
                }
                if undirected_only {
                    continue;
                }
                // 体→头
                if let Some(&head_n) = head_of.get(&node) {
                    if !cuts.contains(&node) {
                        if head_n == forbidden {
                            return None;
                        }
                        if seen.insert(head_n) {
                            queue.push_back(head_n);
                        }
                    }
                }
                // 头→体
                if let Some(&body_n) = body_of_head.get(&node) {
                    if head_of.get(&body_n) == Some(&node) && !cuts.contains(&body_n) {
                        if body_n == forbidden {
                            return None;
                        }
                        if seen.insert(body_n) {
                            queue.push_back(body_n);
                        }
                    }
                }
            }
            let mut nodes: Vec<_> = seen.into_iter().collect();
            nodes.sort_by_key(|id| id.0);
            Some(nodes)
        };

        // (nodes_key, move_dir) → group；同节点且同推动方向才共组
        let mut groups_by_key: HashMap<(Vec<u64>, IVec3), DeformGroup> = HashMap::new();

        let mut bodies: Vec<BlockId> = head_of.keys().copied().collect();
        bodies.sort_by_key(|id| id.0);

        for &body in &bodies {
            let Some(&head) = head_of.get(&body) else {
                continue;
            };
            let Some(&b_fwd) = facing_of.get(&body) else {
                continue;
            };
            let axis = axis_of(b_fwd);

            // 同轴其它推杆：枚举「也切断」子集 → 每条成功传播都是该动作的候选
            let mut peers: Vec<BlockId> = bodies
                .iter()
                .copied()
                .filter(|p| *p != body && facing_of.get(p).is_some_and(|f| axis_of(*f) == axis))
                .collect();
            peers.sort_by_key(|id| id.0);
            if peers.len() > 12 {
                peers.truncate(12);
            }

            let mut candidate_sets: Vec<(bool, Vec<BlockId>)> = Vec::new();
            let peer_n = peers.len();
            let masks = 1u32 << peer_n;
            for mask in 0..masks {
                let mut cuts = HashSet::from([body]);
                for (i, peer) in peers.iter().enumerate() {
                    if mask & (1 << i) != 0 {
                        cuts.insert(*peer);
                    }
                }
                if let Some(nodes) = propagate_with_cuts(head, body, &cuts, false) {
                    if !nodes.is_empty() {
                        candidate_sets.push((true, nodes));
                    }
                }
                if let Some(nodes) = propagate_with_cuts(body, head, &cuts, false) {
                    if !nodes.is_empty() {
                        candidate_sets.push((false, nodes));
                    }
                }
            }

            // 全部同轴单切都成环 → 全切仅无向（对向环）
            let any_fwd = candidate_sets.iter().any(|(f, _)| *f);
            let any_rev = candidate_sets.iter().any(|(f, _)| !*f);
            if !any_fwd || !any_rev {
                let all_cuts: HashSet<BlockId> = head_of.keys().copied().collect();
                if !any_fwd {
                    if let Some(nodes) = propagate_with_cuts(head, body, &all_cuts, true) {
                        if !nodes.is_empty() {
                            candidate_sets.push((true, nodes));
                        }
                    }
                }
                if !any_rev {
                    if let Some(nodes) = propagate_with_cuts(body, head, &all_cuts, true) {
                        if !nodes.is_empty() {
                            candidate_sets.push((false, nodes));
                        }
                    }
                }
            }

            // 正反仍缺一侧则整杆丢弃
            let any_fwd = candidate_sets.iter().any(|(f, _)| *f);
            let any_rev = candidate_sets.iter().any(|(f, _)| !*f);
            if !any_fwd || !any_rev {
                continue;
            }

            for (forward, nodes) in candidate_sets {
                let move_dir = if forward { b_fwd } else { -b_fwd };
                let key = (nodes.iter().map(|id| id.0).collect::<Vec<_>>(), move_dir);
                if let Some(group) = groups_by_key.get_mut(&key) {
                    if !group.actions.contains(&(body, forward)) {
                        group.actions.push((body, forward));
                    }
                } else {
                    groups_by_key.insert(
                        key,
                        DeformGroup {
                            actions: vec![(body, forward)],
                            nodes,
                        },
                    );
                }
            }
        }

        let mut keys: Vec<_> = groups_by_key.keys().cloned().collect();
        keys.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.x.cmp(&b.1.x))
                .then_with(|| a.1.y.cmp(&b.1.y))
                .then_with(|| a.1.z.cmp(&b.1.z))
        });
        let mut deform_groups: Vec<DeformGroup> = Vec::new();
        let mut action_to_groups: HashMap<(BlockId, bool), Vec<u32>> = HashMap::new();
        for key in keys {
            let Some(mut group) = groups_by_key.remove(&key) else {
                continue;
            };
            group.actions.sort_by_key(|(id, fwd)| (id.0, !*fwd));
            let index = deform_groups.len() as u32;
            for action in &group.actions {
                action_to_groups.entry(*action).or_default().push(index);
            }
            deform_groups.push(group);
        }
        for indices in action_to_groups.values_mut() {
            indices.sort_by_key(|idx| deform_groups[*idx as usize].nodes.len());
            indices.dedup();
        }

        if let Some(structure) = self.structures.get_mut(&id) {
            structure.head_of = head_of;
            structure.body_of_head = body_of_head;
            structure.deform_groups = deform_groups;
            structure.action_to_groups = action_to_groups;
            structure.scene_touching = scene_touching;
        }
    }
}
