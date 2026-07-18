//! 一次性将 saves/**/blocks.bin 迁移到当前 VERSION（含 v3→v4 场景 facing 段）

use std::fs;
use std::path::{Path, PathBuf};

use open_infinifactory::shared::platform::SAVE_DIR;
use open_infinifactory::shared::save_format::{
    Cursor, MAGIC, SaveFormatError, VERSION, VERSION_V1, VERSION_V2, VERSION_V3, decode_blocks,
    encode_blocks,
};

fn main() {
    oif_sim::blocks::ensure_fallback_scene_catalog();
    let root = PathBuf::from(SAVE_DIR);
    if !root.is_dir() {
        eprintln!(
            "save dir `{}` not found, nothing to migrate",
            root.display()
        );
        return;
    }
    let mut migrated = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;
    visit(&root, &mut migrated, &mut skipped, &mut failed);
    println!("done: migrated={migrated} skipped={skipped} failed={failed}");
}

fn visit(dir: &Path, migrated: &mut usize, skipped: &mut usize, failed: &mut usize) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit(&path, migrated, skipped, failed);
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) != Some("blocks.bin") {
            continue;
        }
        match migrate_file(&path) {
            Ok(MigrateResult::Migrated) => {
                println!("migrated {}", path.display());
                *migrated += 1;
            }
            Ok(MigrateResult::Skipped) => *skipped += 1,
            Err(err) => {
                eprintln!("failed {}: {err}", path.display());
                *failed += 1;
            }
        }
    }
}

enum MigrateResult {
    Migrated,
    Skipped,
}

fn migrate_file(path: &Path) -> Result<MigrateResult, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let mut cursor = Cursor::new(&bytes);
    if cursor.read_bytes(4).map_err(|e| e.to_string())? != MAGIC {
        return Err(SaveFormatError::InvalidMagic.to_string());
    }
    let version = cursor.read_u16().map_err(|e| e.to_string())?;
    match version {
        VERSION => Ok(MigrateResult::Skipped),
        VERSION_V1 | VERSION_V2 | VERSION_V3 => {
            let data = decode_blocks(&bytes).map_err(|e| e.to_string())?;
            let bak = path.with_extension("bin.bak");
            fs::copy(path, &bak).map_err(|e| format!("backup: {e}"))?;
            let encoded = encode_blocks(&data);
            fs::write(path, encoded).map_err(|e| e.to_string())?;
            Ok(MigrateResult::Migrated)
        }
        other => Err(format!("unsupported version {other}")),
    }
}
