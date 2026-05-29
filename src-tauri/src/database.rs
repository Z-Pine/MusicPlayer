use rusqlite::{Connection, Result as SqlResult};
use std::fs;
use std::path::PathBuf;

use crate::models::*;

/// 获取数据库文件路径
pub fn get_db_path() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("MusicPlayer");
    fs::create_dir_all(&path).ok();
    path.push("musicplayer.db");
    path
}

/// 初始化数据库：建表 + 建索引
pub fn init_db() -> SqlResult<Connection> {
    let db_path = get_db_path();
    let conn = Connection::open(&db_path)?;

    conn.execute_batch(
        r#"
        -- 音乐库源
        CREATE TABLE IF NOT EXISTS library_sources (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            enabled INTEGER DEFAULT 1,
            created_at INTEGER DEFAULT (strftime('%s', 'now'))
        );

        -- 歌曲索引
        CREATE TABLE IF NOT EXISTS songs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            title TEXT,
            artist TEXT,
            album TEXT,
            album_artist TEXT,
            year INTEGER,
            genre TEXT,
            duration INTEGER DEFAULT 0,
            cover_art BLOB,
            bitrate INTEGER,
            sample_rate INTEGER,
            is_available INTEGER DEFAULT 1,
            deleted_from_library INTEGER DEFAULT 0,
            updated_at INTEGER DEFAULT (strftime('%s', 'now'))
        );

        -- 播放列表
        CREATE TABLE IF NOT EXISTS playlists (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            created_at INTEGER DEFAULT (strftime('%s', 'now'))
        );

        -- 播放列表歌曲关联
        CREATE TABLE IF NOT EXISTS playlist_songs (
            playlist_id INTEGER NOT NULL,
            song_id INTEGER NOT NULL,
            position INTEGER DEFAULT 0,
            PRIMARY KEY (playlist_id, song_id),
            FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
            FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE
        );

        -- 最近播放
        CREATE TABLE IF NOT EXISTS recent_plays (
            song_id INTEGER PRIMARY KEY,
            played_at INTEGER DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE
        );

        -- 用户配置
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        );

        -- 播放状态
        CREATE TABLE IF NOT EXISTS playback_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            song_id INTEGER,
            position_ms INTEGER DEFAULT 0,
            volume REAL DEFAULT 0.8,
            play_mode TEXT DEFAULT 'sequence',
            is_playing INTEGER DEFAULT 0,
            FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE SET NULL
        );

        -- 歌词缓存
        CREATE TABLE IF NOT EXISTS lyrics_cache (
            song_id INTEGER PRIMARY KEY,
            source TEXT DEFAULT 'local',
            content TEXT,
            updated_at INTEGER DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE
        );

        -- 索引优化
        CREATE INDEX IF NOT EXISTS idx_songs_artist ON songs(artist);
        CREATE INDEX IF NOT EXISTS idx_songs_album ON songs(album);
        CREATE INDEX IF NOT EXISTS idx_songs_title ON songs(title);
        CREATE INDEX IF NOT EXISTS idx_recent_plays_time ON recent_plays(played_at DESC);
        CREATE INDEX IF NOT EXISTS idx_playlist_songs_pos ON playlist_songs(playlist_id, position);

        -- 初始化默认播放状态
        INSERT OR IGNORE INTO playback_state (id, song_id, position_ms, volume, play_mode, is_playing)
        VALUES (1, NULL, 0, 0.8, 'sequence', 0);
        "#,
    )?;

    // 数据库迁移：为 songs 表添加 deleted_from_library 列（兼容旧版数据库）
    let has_column: bool = conn
        .prepare("PRAGMA table_info(songs)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .any(|name| name.as_deref() == Ok("deleted_from_library"));
    if !has_column {
        conn.execute_batch("ALTER TABLE songs ADD COLUMN deleted_from_library INTEGER DEFAULT 0;")?;
    }

    Ok(conn)
}

// ==================== LibrarySources ====================

/// 添加音乐库源
pub fn add_library_source(conn: &Connection, path: &str) -> SqlResult<i64> {
    conn.execute(
        "INSERT OR IGNORE INTO library_sources (path) VALUES (?1)",
        [path],
    )?;
    Ok(conn.last_insert_rowid())
}

/// 删除音乐库源
pub fn remove_library_source(conn: &Connection, id: i64) -> SqlResult<()> {
    conn.execute("DELETE FROM library_sources WHERE id = ?1", [id])?;
    Ok(())
}

/// 获取所有音乐库源
pub fn get_library_sources(conn: &Connection) -> SqlResult<Vec<LibrarySource>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, enabled, created_at FROM library_sources ORDER BY created_at"
    )?;
    let sources = stmt.query_map([], |row| {
        Ok(LibrarySource {
            id: row.get(0)?,
            path: row.get(1)?,
            enabled: row.get::<_, i32>(2)? != 0,
            created_at: row.get(3)?,
        })
    })?;
    sources.collect()
}

// ==================== Songs ====================

/// 插入或更新歌曲
pub fn upsert_song(conn: &Connection, song: &Song) -> SqlResult<i64> {
    conn.execute(
        r#"INSERT INTO songs (path, title, artist, album, album_artist, year, genre, duration, cover_art, bitrate, sample_rate, is_available, deleted_from_library, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
         ON CONFLICT(path) DO UPDATE SET
         title=excluded.title, artist=excluded.artist, album=excluded.album,
         album_artist=excluded.album_artist, year=excluded.year, genre=excluded.genre,
         duration=excluded.duration, cover_art=excluded.cover_art, bitrate=excluded.bitrate,
         sample_rate=excluded.sample_rate, is_available=1, deleted_from_library=0, updated_at=excluded.updated_at"#,
        rusqlite::params![
            song.path, song.title, song.artist, song.album, song.album_artist,
            song.year, song.genre, song.duration, song.cover_art, song.bitrate,
            song.sample_rate, song.is_available as i32, song.deleted_from_library as i32, song.updated_at
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// 根据ID获取歌曲
pub fn get_song_by_id(conn: &Connection, id: i64) -> SqlResult<Option<Song>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, title, artist, album, album_artist, year, genre, duration, cover_art, bitrate, sample_rate, is_available, deleted_from_library, updated_at FROM songs WHERE id = ?1"
    )?;
    let mut rows = stmt.query([id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(map_song_row(row)?))
    } else {
        Ok(None)
    }
}

/// 获取所有歌曲（不含 cover_art BLOB，提升查询速度）
pub fn get_all_songs(conn: &Connection) -> SqlResult<Vec<Song>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, title, artist, album, album_artist, year, genre, duration, NULL as cover_art, bitrate, sample_rate, is_available, deleted_from_library, updated_at FROM songs WHERE deleted_from_library = 0 ORDER BY title"
    )?;
    let songs = stmt.query_map([], |row| map_song_row(row))?;
    songs.collect()
}

/// 获取所有歌曲（包括已从音乐库删除的，用于歌单展示）
pub fn get_all_songs_with_deleted(conn: &Connection) -> SqlResult<Vec<Song>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, title, artist, album, album_artist, year, genre, duration, NULL as cover_art, bitrate, sample_rate, is_available, deleted_from_library, updated_at FROM songs ORDER BY title"
    )?;
    let songs = stmt.query_map([], |row| map_song_row(row))?;
    songs.collect()
}

/// 获取所有歌曲（仅基础字段，不含 cover_art，用于弹窗选择）
pub fn get_all_songs_basic(conn: &Connection) -> SqlResult<Vec<Song>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, title, artist, album, album_artist, year, genre, duration, NULL, bitrate, sample_rate, is_available, deleted_from_library, updated_at FROM songs WHERE deleted_from_library = 0 ORDER BY title"
    )?;
    let songs = stmt.query_map([], |row| map_song_row(row))?;
    songs.collect()
}

/// 搜索歌曲（仅基础字段，不含 cover_art，用于弹窗选择）
pub fn search_songs_basic(conn: &Connection, keyword: &str) -> SqlResult<Vec<Song>> {
    let pattern = format!("%{}%", keyword);
    let mut stmt = conn.prepare(
        "SELECT id, path, title, artist, album, album_artist, year, genre, duration, NULL, bitrate, sample_rate, is_available, deleted_from_library, updated_at FROM songs
         WHERE deleted_from_library = 0 AND (title LIKE ?1 OR artist LIKE ?1 OR album LIKE ?1)
         ORDER BY title"
    )?;
    let songs = stmt.query_map([&pattern], |row| map_song_row(row))?;
    songs.collect()
}

/// 搜索歌曲
pub fn search_songs(conn: &Connection, keyword: &str) -> SqlResult<Vec<Song>> {
    let pattern = format!("%{}%", keyword);
    let mut stmt = conn.prepare(
        "SELECT id, path, title, artist, album, album_artist, year, genre, duration, cover_art, bitrate, sample_rate, is_available, deleted_from_library, updated_at FROM songs
         WHERE deleted_from_library = 0 AND (title LIKE ?1 OR artist LIKE ?1 OR album LIKE ?1)
         ORDER BY title"
    )?;
    let songs = stmt.query_map([&pattern], |row| map_song_row(row))?;
    songs.collect()
}

/// 标记歌曲为不可用（文件被删除时）
pub fn mark_song_unavailable(conn: &Connection, path: &str) -> SqlResult<()> {
    conn.execute(
        "UPDATE songs SET is_available = 0 WHERE path = ?1",
        [path],
    )?;
    Ok(())
}

/// 从音乐库删除歌曲（标记 deleted_from_library，不从磁盘删除）
pub fn delete_song_from_library(conn: &Connection, id: i64) -> SqlResult<()> {
    conn.execute(
        "UPDATE songs SET deleted_from_library = 1 WHERE id = ?1",
        [id],
    )?;
    Ok(())
}

/// 批量从音乐库删除歌曲
pub fn batch_delete_songs_from_library(conn: &Connection, ids: &[i64]) -> SqlResult<usize> {
    if ids.is_empty() {
        return Ok(0);
    }
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!("UPDATE songs SET deleted_from_library = 1 WHERE id IN ({}) AND deleted_from_library = 0", placeholders);
    let mut stmt = conn.prepare(&sql)?;
    let params: Vec<&dyn rusqlite::ToSql> = ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
    let count = stmt.execute(&*params)?;
    Ok(count)
}

/// 获取所有无效歌曲（文件不可用或已从音乐库删除）
pub fn get_invalid_songs(conn: &Connection) -> SqlResult<Vec<Song>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, title, artist, album, album_artist, year, genre, duration, NULL as cover_art, bitrate, sample_rate, is_available, deleted_from_library, updated_at FROM songs
         WHERE is_available = 0 OR deleted_from_library = 1"
    )?;
    let songs = stmt.query_map([], |row| map_song_row(row))?;
    songs.collect()
}

/// 一键清理所有无效歌曲（从数据库中物理删除）
pub fn cleanup_invalid_songs(conn: &Connection) -> SqlResult<usize> {
    let count = conn.execute("DELETE FROM songs WHERE is_available = 0 OR deleted_from_library = 1", [])?;
    Ok(count)
}

/// 检查歌曲文件是否存在（返回不存在的歌曲路径列表）
pub fn check_songs_file_exists(conn: &Connection) -> SqlResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT path FROM songs WHERE deleted_from_library = 0")?;
    let paths: Vec<String> = stmt.query_map([], |row| row.get::<_, String>(0))?.collect::<SqlResult<Vec<_>>>()?;
    
    let mut missing_paths = Vec::new();
    for path in paths {
        if !std::path::Path::new(&path).exists() {
            missing_paths.push(path);
        }
    }
    Ok(missing_paths)
}

/// 获取歌单中已失效的歌曲数量
pub fn get_playlist_invalid_song_count(conn: &Connection, playlist_id: i64) -> SqlResult<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM playlist_songs ps
         JOIN songs s ON ps.song_id = s.id
         WHERE ps.playlist_id = ?1 AND (s.is_available = 0 OR s.deleted_from_library = 1)",
        [playlist_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// 清理歌单中已失效的歌曲
pub fn cleanup_playlist_invalid_songs(conn: &Connection, playlist_id: i64) -> SqlResult<usize> {
    let count = conn.execute(
        "DELETE FROM playlist_songs WHERE playlist_id = ?1 AND song_id IN (
            SELECT id FROM songs WHERE is_available = 0 OR deleted_from_library = 1
        )",
        [playlist_id],
    )?;
    Ok(count)
}

/// 获取所有歌单及其失效歌曲数量
pub fn get_playlists_with_invalid_counts(conn: &Connection) -> SqlResult<Vec<(Playlist, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name, p.created_at,
                COUNT(CASE WHEN s.is_available = 0 OR s.deleted_from_library = 1 THEN 1 END) as invalid_count
         FROM playlists p
         LEFT JOIN playlist_songs ps ON p.id = ps.playlist_id
         LEFT JOIN songs s ON ps.song_id = s.id
         GROUP BY p.id, p.name, p.created_at
         ORDER BY p.created_at"
    )?;
    let results = stmt.query_map([], |row| {
        Ok((
            Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            },
            row.get(3)?,
        ))
    })?;
    results.collect()
}

fn map_song_row(row: &rusqlite::Row) -> SqlResult<Song> {
    Ok(Song {
        id: row.get(0)?,
        path: row.get(1)?,
        title: row.get(2)?,
        artist: row.get(3)?,
        album: row.get(4)?,
        album_artist: row.get(5)?,
        year: row.get(6)?,
        genre: row.get(7)?,
        duration: row.get(8)?,
        cover_art: row.get(9)?,
        bitrate: row.get(10)?,
        sample_rate: row.get(11)?,
        is_available: row.get::<_, i32>(12)? != 0,
        deleted_from_library: row.get::<_, i32>(13)? != 0,
        updated_at: row.get(14)?,
    })
}

/// 批量检查文件完整性：遍历所有非删除歌曲，标记缺失/恢复的文件
pub fn check_and_mark_invalid_files(conn: &Connection) -> rusqlite::Result<(i64, i64, i64)> {
    let mut stmt = conn.prepare(
        "SELECT id, path FROM songs WHERE deleted_from_library = 0"
    )?;
    let paths: Vec<(i64, String)> = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?.collect::<rusqlite::Result<Vec<_>>>()?;

    let mut marked_missing: i64 = 0;
    let marked_restored: i64 = 0;

    for (id, path) in &paths {
        let exists = std::path::Path::new(path).exists();
        // 标记缺失文件
        conn.execute(
            "UPDATE songs SET is_available = ?1 WHERE id = ?2 AND deleted_from_library = 0",
            rusqlite::params![exists as i32, id],
        )?;
        if !exists {
            marked_missing += 1;
        }
    }

    let total = paths.len() as i64;
    Ok((total, marked_missing, marked_restored))
}

// ==================== Playlists ====================

/// 创建播放列表
pub fn create_playlist(conn: &Connection, name: &str) -> SqlResult<i64> {
    conn.execute("INSERT INTO playlists (name) VALUES (?1)", [name])?;
    Ok(conn.last_insert_rowid())
}

/// 重命名播放列表
pub fn rename_playlist(conn: &Connection, id: i64, name: &str) -> SqlResult<()> {
    conn.execute("UPDATE playlists SET name = ?1 WHERE id = ?2", [name, &id.to_string()])?;
    Ok(())
}

/// 删除播放列表（先删除关联歌曲，再删除歌单本身）
pub fn delete_playlist(conn: &Connection, id: i64) -> SqlResult<()> {
    conn.execute("DELETE FROM playlist_songs WHERE playlist_id = ?1", [id])?;
    conn.execute("DELETE FROM playlists WHERE id = ?1", [id])?;
    Ok(())
}

/// 获取所有播放列表
pub fn get_playlists(conn: &Connection) -> SqlResult<Vec<Playlist>> {
    let mut stmt = conn.prepare("SELECT id, name, created_at FROM playlists ORDER BY created_at")?;
    let playlists = stmt.query_map([], |row| {
        Ok(Playlist {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        })
    })?;
    playlists.collect()
}

/// 添加歌曲到播放列表
pub fn add_song_to_playlist(conn: &Connection, playlist_id: i64, song_id: i64) -> SqlResult<()> {
    // 获取当前最大position
    let max_pos: i32 = conn.query_row(
        "SELECT COALESCE(MAX(position), 0) FROM playlist_songs WHERE playlist_id = ?1",
        [playlist_id],
        |row| row.get(0),
    ).unwrap_or(0);

    conn.execute(
        "INSERT OR IGNORE INTO playlist_songs (playlist_id, song_id, position) VALUES (?1, ?2, ?3)",
        [playlist_id, song_id, (max_pos + 1) as i64],
    )?;
    Ok(())
}

/// 从播放列表移除歌曲
pub fn remove_song_from_playlist(conn: &Connection, playlist_id: i64, song_id: i64) -> SqlResult<()> {
    conn.execute(
        "DELETE FROM playlist_songs WHERE playlist_id = ?1 AND song_id = ?2",
        [playlist_id, song_id],
    )?;
    Ok(())
}

/// 获取播放列表中的歌曲
pub fn get_playlist_songs(conn: &Connection, playlist_id: i64) -> SqlResult<Vec<PlaylistSong>> {
    let mut stmt = conn.prepare(
        "SELECT ps.playlist_id, ps.song_id, ps.position,
                s.id, s.path, s.title, s.artist, s.album, s.album_artist, s.year, s.genre, s.duration, NULL as cover_art, s.bitrate, s.sample_rate, s.is_available, s.deleted_from_library, s.updated_at
         FROM playlist_songs ps
         JOIN songs s ON ps.song_id = s.id
         WHERE ps.playlist_id = ?1
         ORDER BY ps.position"
    )?;
    let songs = stmt.query_map([playlist_id], |row| {
        let song = Song {
            id: row.get(3)?,
            path: row.get(4)?,
            title: row.get(5)?,
            artist: row.get(6)?,
            album: row.get(7)?,
            album_artist: row.get(8)?,
            year: row.get(9)?,
            genre: row.get(10)?,
            duration: row.get(11)?,
            cover_art: row.get(12)?,
            bitrate: row.get(13)?,
            sample_rate: row.get(14)?,
            is_available: row.get::<_, i32>(15)? != 0,
            deleted_from_library: row.get::<_, i32>(16)? != 0,
            updated_at: row.get(17)?,
        };
        Ok(PlaylistSong {
            playlist_id: row.get(0)?,
            song_id: row.get(1)?,
            position: row.get(2)?,
            song: Some(song),
        })
    })?;
    songs.collect()
}

// ==================== Recent Plays ====================

/// 记录最近播放
pub fn record_recent_play(conn: &Connection, song_id: i64) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO recent_plays (song_id, played_at) VALUES (?1, strftime('%s', 'now'))
        ON CONFLICT(song_id) DO UPDATE SET played_at = excluded.played_at",
        [song_id],
    )?;
    // 清理超出100条的记录
    conn.execute(
        "DELETE FROM recent_plays WHERE song_id NOT IN (
            SELECT song_id FROM recent_plays ORDER BY played_at DESC LIMIT 100
        )",
        [],
    )?;
    Ok(())
}

/// 获取最近播放
pub fn get_recent_plays(conn: &Connection, limit: i64) -> SqlResult<Vec<RecentPlay>> {
    let mut stmt = conn.prepare(
        "SELECT r.song_id, r.played_at,
                s.id, s.path, s.title, s.artist, s.album, s.album_artist, s.year, s.genre, s.duration, NULL as cover_art, s.bitrate, s.sample_rate, s.is_available, s.deleted_from_library, s.updated_at
         FROM recent_plays r
         JOIN songs s ON r.song_id = s.id
         ORDER BY r.played_at DESC
         LIMIT ?1"
    )?;
    let plays = stmt.query_map([limit], |row| {
        let song = Song {
            id: row.get(2)?,
            path: row.get(3)?,
            title: row.get(4)?,
            artist: row.get(5)?,
            album: row.get(6)?,
            album_artist: row.get(7)?,
            year: row.get(8)?,
            genre: row.get(9)?,
            duration: row.get(10)?,
            cover_art: row.get(11)?,
            bitrate: row.get(12)?,
            sample_rate: row.get(13)?,
            is_available: row.get::<_, i32>(14)? != 0,
            deleted_from_library: row.get::<_, i32>(15)? != 0,
            updated_at: row.get(16)?,
        };
        Ok(RecentPlay {
            song_id: row.get(0)?,
            played_at: row.get(1)?,
            song: Some(song),
        })
    })?;
    plays.collect()
}

// ==================== Settings ====================

/// 设置配置项
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [key, value],
    )?;
    Ok(())
}

/// 获取配置项
pub fn get_setting(conn: &Connection, key: &str) -> SqlResult<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query([key])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

// ==================== Playback State ====================

/// 保存播放状态
pub fn save_playback_state(conn: &Connection, state: &PlaybackState) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO playback_state (id, song_id, position_ms, volume, play_mode, is_playing)
         VALUES (1, ?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET
         song_id=excluded.song_id, position_ms=excluded.position_ms,
         volume=excluded.volume, play_mode=excluded.play_mode, is_playing=excluded.is_playing",
        rusqlite::params![
            state.song_id, state.position_ms, state.volume, state.play_mode, state.is_playing as i32
        ],
    )?;
    Ok(())
}

/// 读取播放状态
pub fn load_playback_state(conn: &Connection) -> SqlResult<PlaybackState> {
    conn.query_row(
        "SELECT song_id, position_ms, volume, play_mode, is_playing FROM playback_state WHERE id = 1",
        [],
        |row| {
            Ok(PlaybackState {
                song_id: row.get(0)?,
                position_ms: row.get(1)?,
                volume: row.get(2)?,
                play_mode: row.get(3)?,
                is_playing: row.get::<_, i32>(4)? != 0,
            })
        },
    )
}

// ==================== Lyrics Cache ====================

/// 缓存歌词
pub fn cache_lyrics(conn: &Connection, song_id: i64, source: &str, content: &str) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO lyrics_cache (song_id, source, content, updated_at)
         VALUES (?1, ?2, ?3, strftime('%s', 'now'))
         ON CONFLICT(song_id) DO UPDATE SET source=excluded.source, content=excluded.content, updated_at=excluded.updated_at",
        rusqlite::params![song_id, source, content],
    )?;
    Ok(())
}

/// 获取缓存歌词
pub fn get_cached_lyrics(conn: &Connection, song_id: i64) -> SqlResult<Option<(String, String)>> {
    let mut stmt = conn.prepare("SELECT source, content FROM lyrics_cache WHERE song_id = ?1")?;
    let mut rows = stmt.query([song_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some((row.get(0)?, row.get(1)?)))
    } else {
        Ok(None)
    }
}
