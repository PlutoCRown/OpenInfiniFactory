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
    for free in persistent_storage::list_frees() {
        let slot = SaveSlot::free(&free);
        entries.push(SaveEntry {
            name: entry_display_name(&slot, &free),
            slot,
            kind: SaveKind::Free,
        });
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

/// Puzzle + Free 顶层名字（共享命名空间）
pub fn top_level_world_names(entries: &[SaveEntry]) -> Vec<String> {
    entries
        .iter()
        .filter(|entry| matches!(entry.kind, SaveKind::Puzzle | SaveKind::Free))
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
