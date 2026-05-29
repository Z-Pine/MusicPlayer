use serde::{Deserialize, Serialize};

/// 音乐库源
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LibrarySource {
    pub id: i64,
    pub path: String,
    pub enabled: bool,
    pub created_at: i64,
}

/// 歌曲
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: i64,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub year: Option<i32>,
    pub genre: String,
    pub duration: i64,
    pub cover_art: Option<Vec<u8>>,
    pub bitrate: Option<i32>,
    pub sample_rate: Option<i32>,
    pub is_available: bool,
    pub deleted_from_library: bool,
    pub updated_at: i64,
}

/// 播放列表
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub created_at: i64,
}

/// 播放列表中的歌曲（带顺序）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistSong {
    pub playlist_id: i64,
    pub song_id: i64,
    pub position: i32,
    pub song: Option<Song>,
}

/// 最近播放
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecentPlay {
    pub song_id: i64,
    pub played_at: i64,
    pub song: Option<Song>,
}

/// 用户配置项
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    pub key: String,
    pub value: String,
}

/// 播放状态（用于恢复）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackState {
    pub song_id: Option<i64>,
    pub position_ms: i64,
    pub volume: f32,
    pub play_mode: String, // "sequence" | "loop" | "single_loop" | "shuffle"
    pub is_playing: bool,
}

/// 歌词行
#[derive(Debug, Serialize, Clone)]
pub struct LyricLine {
    pub time: f64,
    pub text: String,
}

/// 文件完整性检查结果
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CheckResult {
    pub checked: i64,
    pub marked_missing: i64,
    pub marked_restored: i64,
}

/// 文件信息
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub size: i64,
    pub modified: Option<i64>,
    pub created: Option<i64>,
}
