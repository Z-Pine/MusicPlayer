use tauri::State;
use std::sync::Mutex;
use rusqlite::Connection;
use lofty::file::{TaggedFileExt, AudioFile};
use lofty::tag::Accessor;

use crate::database;
use crate::models::*;

/// 共享的 AppState，包含数据库连接
pub struct AppState {
    pub db: Mutex<Connection>,
}

// ==================== Library Sources Commands ====================

#[tauri::command]
pub fn add_library_source(state: State<AppState>, path: String) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::add_library_source(&conn, &path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_library_source(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::remove_library_source(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_library_sources(state: State<AppState>) -> Result<Vec<LibrarySource>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::get_library_sources(&conn).map_err(|e| e.to_string())
}

// ==================== Songs Commands ====================

#[tauri::command]
pub fn get_all_songs(state: State<AppState>) -> Result<Vec<Song>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::get_all_songs(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_songs_basic(state: State<AppState>) -> Result<Vec<Song>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::get_all_songs_basic(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_song_by_id(state: State<AppState>, id: i64) -> Result<Option<Song>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::get_song_by_id(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_songs(state: State<AppState>, keyword: String) -> Result<Vec<Song>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::search_songs(&conn, &keyword).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_song_unavailable(state: State<AppState>, path: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::mark_song_unavailable(&conn, &path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_song_from_library(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::delete_song_from_library(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn batch_delete_songs_from_library(state: State<AppState>, ids: Vec<i64>) -> Result<usize, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::batch_delete_songs_from_library(&conn, &ids).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_invalid_songs(state: State<AppState>) -> Result<Vec<Song>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::get_invalid_songs(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cleanup_invalid_songs(state: State<AppState>) -> Result<usize, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::cleanup_invalid_songs(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_and_mark_invalid_files(state: State<AppState>) -> Result<CheckResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let (checked, marked_missing, marked_restored) = database::check_and_mark_invalid_files(&conn)
        .map_err(|e| e.to_string())?;
    Ok(CheckResult { checked, marked_missing, marked_restored })
}

#[tauri::command]
pub fn check_songs_file_exists(state: State<AppState>) -> Result<Vec<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::check_songs_file_exists(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cleanup_playlist_invalid_songs(state: State<AppState>, playlistId: i64) -> Result<usize, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::cleanup_playlist_invalid_songs(&conn, playlistId).map_err(|e| e.to_string())
}

// ==================== Playlists Commands ====================

#[tauri::command]
pub fn create_playlist(state: State<AppState>, name: String) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::create_playlist(&conn, &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_playlist(state: State<AppState>, id: i64, name: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::rename_playlist(&conn, id, &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_playlist(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::delete_playlist(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_playlists(state: State<AppState>) -> Result<Vec<Playlist>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::get_playlists(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_song_to_playlist(state: State<AppState>, playlistId: i64, songId: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::add_song_to_playlist(&conn, playlistId, songId).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_song_from_playlist(state: State<AppState>, playlistId: i64, songId: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::remove_song_from_playlist(&conn, playlistId, songId).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_playlist_songs(state: State<AppState>, playlistId: i64) -> Result<Vec<PlaylistSong>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::get_playlist_songs(&conn, playlistId).map_err(|e| e.to_string())
}

// ==================== Recent Plays Commands ====================

#[tauri::command]
pub fn record_recent_play(state: State<AppState>, songId: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::record_recent_play(&conn, songId).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_recent_plays(state: State<AppState>, limit: i64) -> Result<Vec<RecentPlay>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::get_recent_plays(&conn, limit).map_err(|e| e.to_string())
}

// ==================== Settings Commands ====================

#[tauri::command]
pub fn set_setting(state: State<AppState>, key: String, value: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::set_setting(&conn, &key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_setting(state: State<AppState>, key: String) -> Result<Option<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::get_setting(&conn, &key).map_err(|e| e.to_string())
}

// ==================== Playback State Commands ====================

#[tauri::command]
pub fn save_playback_state_cmd(state: State<AppState>, state_data: PlaybackState) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::save_playback_state(&conn, &state_data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_playback_state_cmd(state: State<AppState>) -> Result<PlaybackState, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::load_playback_state(&conn).map_err(|e| e.to_string())
}

// ==================== Lyrics Cache Commands ====================

#[tauri::command]
pub fn cache_lyrics_cmd(state: State<AppState>, song_id: i64, source: String, content: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::cache_lyrics(&conn, song_id, &source, &content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_cached_lyrics_cmd(state: State<AppState>, song_id: i64) -> Result<Option<(String, String)>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::get_cached_lyrics(&conn, song_id).map_err(|e| e.to_string())
}

// ==================== Lyrics Commands ====================

use std::fs;
use std::path::Path;
use encoding_rs::{UTF_8, GB18030};

/// 读取文件内容，自动检测编码（UTF-8 → GB18030）
fn read_file_with_encoding(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    
    // 先尝试 UTF-8
    if let Ok(content) = String::from_utf8(bytes.clone()) {
        return Some(content);
    }
    
    // 尝试 GB18030（兼容 GBK、GB2312）
    let (cow, _, had_errors) = GB18030.decode(&bytes);
    if !had_errors {
        return Some(cow.into_owned());
    }
    
    // 最后一次尝试：用 UTF-8 的 lossy 解码
    Some(UTF_8.decode(&bytes).0.into_owned())
}

/// 解析 LRC 歌词文件
fn parse_lrc(content: &str) -> Vec<LyricLine> {
    let mut lines = Vec::new();
    let re = regex::Regex::new(r"\[(\d{2}):(\d{2})(?:\.(\d{2,3}))?\](.*)").unwrap();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // 一行可能有多个时间标签，如 [00:01.23][00:02.34]歌词
        let mut times = Vec::new();
        let mut text = String::new();

        for cap in re.captures_iter(line) {
            let min: f64 = cap[1].parse().unwrap_or(0.0);
            let sec: f64 = cap[2].parse().unwrap_or(0.0);
            let ms: f64 = cap.get(3).map(|m| {
                let s = m.as_str();
                if s.len() == 2 {
                    s.parse::<f64>().unwrap_or(0.0) / 100.0
                } else {
                    s.parse::<f64>().unwrap_or(0.0) / 1000.0
                }
            }).unwrap_or(0.0);
            let time = min * 60.0 + sec + ms;
            let t = cap.get(4).map(|m| m.as_str().trim()).unwrap_or("").to_string();

            times.push(time);
            if text.is_empty() {
                text = t;
            }
        }

        for time in times {
            lines.push(LyricLine { time, text: text.clone() });
        }
    }

    lines.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    lines
}

/// 尝试从本地获取歌词：
/// 1. 先查找同目录下同名的 .lrc 文件
/// 2. 如果没有，返回空列表
#[tauri::command]
pub fn get_lyrics(state: State<AppState>, song_id: i64) -> Result<Vec<LyricLine>, String> {
    let song = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        database::get_song_by_id(&conn, song_id).map_err(|e| e.to_string())?
    };

    let song = match song {
        Some(s) => s,
        None => return Ok(Vec::new()),
    };

    let song_path = Path::new(&song.path);
    let parent = song_path.parent().unwrap_or(Path::new("."));
    let stem = song_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    // 尝试加载 .lrc 文件（支持 UTF-8 / GBK / GB2312 编码自动检测）
    let lrc_path = parent.join(format!("{}.lrc", stem));
    if lrc_path.exists() {
        if let Some(content) = read_file_with_encoding(&lrc_path) {
            return Ok(parse_lrc(&content));
        }
    }

    // 尝试从缓存加载（如果有的话）
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        if let Ok(Some((_, content))) = database::get_cached_lyrics(&conn, song_id) {
            return Ok(parse_lrc(&content));
        }
    }

    Ok(Vec::new())
}

// ==================== File Info Command ====================

#[tauri::command]
pub fn get_file_info(path: String) -> Result<FileInfo, String> {
    let metadata = std::fs::metadata(&path).map_err(|e| format!("无法读取文件: {}", e))?;
    Ok(FileInfo {
        size: metadata.len() as i64,
        modified: metadata.modified().ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64),
        created: metadata.created().ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64),
    })
}

#[tauri::command]
pub fn open_file_location(path: String) -> Result<(), String> {
    // Windows: 用 explorer /select 打开文件夹并选中该文件
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("无法打开文件位置: {}", e))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Some(parent) = std::path::Path::new(&path).parent() {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("无法打开文件位置: {}", e))?;
        }
    }
    Ok(())
}

// ==================== Scanner Commands ====================

use walkdir::WalkDir;
use std::time::{SystemTime, UNIX_EPOCH};

#[tauri::command]
pub fn scan_library(state: State<AppState>) -> Result<i64, String> {
    let sources = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        database::get_library_sources(&conn).map_err(|e| e.to_string())?
    };

    let mut scanned_count: i64 = 0;
    let extensions = ["mp3", "flac", "wav", "aac", "m4a", "ogg", "wma"];

    for source in sources {
        if !source.enabled {
            continue;
        }
        for entry in WalkDir::new(&source.path).follow_links(true) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if !extensions.contains(&ext.as_str()) {
                continue;
            }

            let path_str = path.to_string_lossy().to_string();
            let mut song = Song {
                id: 0,
                path: path_str.clone(),
                title: path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string(),
                artist: String::new(),
                album: String::new(),
                album_artist: String::new(),
                year: None,
                genre: String::new(),
                duration: 0,
                cover_art: None,
                bitrate: None,
                sample_rate: None,
                is_available: true,
                deleted_from_library: false,
                updated_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
            };

            // 尝试解析元数据
            if let Ok(tagged_file) = lofty::read_from_path(path) {
                if let Some(tag) = tagged_file.primary_tag() {
                    song.title = tag.title().as_ref().map(|t| t.to_string()).unwrap_or_else(|| song.title.clone());
                    song.artist = tag.artist().as_ref().map(|a| a.to_string()).unwrap_or_default();
                    song.album = tag.album().as_ref().map(|a| a.to_string()).unwrap_or_default();
                    song.genre = tag.genre().as_ref().map(|g| g.to_string()).unwrap_or_default();
                    // 尝试获取封面
                    if let Some(picture) = tag.pictures().first() {
                        song.cover_art = Some(picture.data().to_vec());
                    }
                }
                let properties = tagged_file.properties();
                song.duration = properties.duration().as_secs() as i64;
                song.bitrate = properties.audio_bitrate().map(|b| b as i32);
                song.sample_rate = properties.sample_rate().map(|s| s as i32);
            }

            let conn = state.db.lock().map_err(|e| e.to_string())?;
            if database::upsert_song(&conn, &song).is_ok() {
                scanned_count += 1;
            }
        }
    }

    Ok(scanned_count)
}
