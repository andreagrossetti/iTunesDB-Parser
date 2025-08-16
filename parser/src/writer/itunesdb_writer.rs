use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::helpers::helpers as io_helpers;
use crate::itunesdb::Song;

/// Minimal iTunesDB writer scaffold.
/// NOTE: This currently writes a minimal MHBD header only, as a placeholder.
/// We'll extend this to write MHIT/MHOD entries and optional playlists.
pub fn write_itunesdb_from_json(songs_json_path: &str, dest_path: &str) -> std::io::Result<()> {
    // Read songs for planning (count only for now)
    let songs: Vec<Song> = match std::fs::read_to_string(songs_json_path) {
        Ok(s) => match serde_json::from_str(&s) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[writer] Failed to parse JSON at '{}': {}. Proceeding with 0 songs.", songs_json_path, e);
                Vec::new()
            }
        },
        Err(e) => {
            eprintln!("[writer] Could not read '{}': {}. Proceeding with 0 songs.", songs_json_path, e);
            Vec::new()
        }
    };

    eprintln!(
        "[writer] Writing placeholder iTunesDB to {} ({} songs in plan)",
        dest_path,
        songs.len()
    );

    // Write to temp, then atomic rename
    let dest = Path::new(dest_path);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp_path = dest.with_extension("tmp");

    let file = File::create(&tmp_path)?;
    let mut w = BufWriter::new(file);

    // --- MHBD (Database header) minimal placeholder ---
    // 'mhbd' (4 bytes)
    w.write_all(b"mhbd")?;

    // total length (u32 LE) -> placeholder 28 bytes
    io_helpers::write_le_u32(&mut w, 28)?;

    // version or flags (u32 LE) placeholder
    io_helpers::write_le_u32(&mut w, 0x0000_0001)?;

    // reserved / timestamps placeholders (3 * u32)
    io_helpers::write_le_u32(&mut w, 0)?;
    io_helpers::write_le_u32(&mut w, 0)?;
    io_helpers::write_le_u32(&mut w, 0)?;

    // padding to 28 bytes if needed
    io_helpers::pad_to_4(&mut w, 28)?;

    w.flush()?;

    // fsync and atomic rename
    // On macOS, BufWriter does not expose sync_all directly; re-open for sync
    drop(w);
    let f = File::options().read(true).write(true).open(&tmp_path)?;
    f.sync_all()?;
    drop(f);

    std::fs::rename(&tmp_path, &dest)?;

    eprintln!("[writer] Wrote placeholder iTunesDB to {}", dest_path);
    Ok(())
}
