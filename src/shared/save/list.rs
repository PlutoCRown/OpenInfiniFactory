pub fn list_saves() -> Vec<String> {
    persistent_storage::list_saves()
}

pub fn list_save_entries() -> Vec<SaveEntry> {
    let mut entries = Vec::new();
    for puzzle in persistent_storage::list_puzzles() {
        let slot = SaveSlot::puzzle(&puzzle);
        entries.push(entry_from_slot(slot, &puzzle, SaveKind::Puzzle));
        for solution in persistent_storage::list_solution_names(&puzzle) {
            let slot = SaveSlot::solution(&puzzle, &solution);
            entries.push(entry_from_slot(slot, &solution, SaveKind::Solution));
        }
    }
    for free in persistent_storage::list_frees() {
        let slot = SaveSlot::free(&free);
        entries.push(entry_from_slot(slot, &free, SaveKind::Free));
    }
    // 创建时间新→旧；无时间戳的旧档排后面，再按名字
    entries.sort_by(|a, b| {
        b.created_at
            .cmp(&a.created_at)
            .then(a.name.cmp(&b.name))
            .then(a.slot.puzzle.cmp(&b.slot.puzzle))
    });
    entries
}

fn entry_from_slot(slot: SaveSlot, fallback: &str, kind: SaveKind) -> SaveEntry {
    let (name, created_at, updated_at, last_play_time, favorite) = read_save(&slot)
        .map(|save| {
            (
                save.meta
                    .name
                    .filter(|n| !n.trim().is_empty())
                    .unwrap_or_else(|| fallback.to_string()),
                save.meta.created_at,
                save.meta.updated_at,
                save.meta.last_play_time,
                save.meta.favorite,
            )
        })
        .unwrap_or_else(|| (fallback.to_string(), None, None, None, false));
    SaveEntry {
        slot,
        name,
        kind,
        created_at,
        updated_at,
        last_play_time,
        favorite,
    }
}

/// 读 meta.name；没有则用文件夹名
fn read_save_name(slot: &SaveSlot) -> Option<String> {
    read_save(slot)
        .and_then(|save| save.meta.name)
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
}

/// 相对上次保存时间：`N min ago` / `N hr ago` / `N day ago`
pub fn format_relative_saved_at(updated_at: u64, now: u64) -> String {
    let secs = now.saturating_sub(updated_at);
    let mins = secs / 60;
    if mins < 60 {
        format!("{mins} min ago")
    } else {
        let hrs = mins / 60;
        if hrs < 24 {
            format!("{hrs} hr ago")
        } else {
            format!("{} day ago", hrs / 24)
        }
    }
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
