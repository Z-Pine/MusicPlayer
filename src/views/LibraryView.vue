<template>
  <div class="library-view">
    <!-- 音乐库横幅 -->
    <header class="library-header-compact">
      <div class="header-left">
        <div class="library-icon-small">🏠</div>
        <div>
          <h1 class="library-title-compact">音乐库</h1>
          <p class="library-subtitle">你的个人音乐收藏</p>
        </div>
      </div>
      <div class="header-right">
        <div class="search-box-inline">
          <span class="search-icon">🔍</span>
          <input
            v-model="searchKeyword"
            type="text"
            class="search-input-inline"
            placeholder="搜索歌曲、艺术家、专辑..."
            @input="onSearch"
          />
        </div>
        <button class="action-btn-compact play-all" @click="playAll" v-if="songs.length > 0">
          <span>▶</span>
          <span>播放全部</span>
        </button>
        <button class="action-btn-compact shuffle" @click="shufflePlay" v-if="songs.length > 0">
          <span>🔀</span>
          <span>随机</span>
        </button>
        <button class="action-btn-compact scan" @click="scanLibrary" :disabled="scanning">
          <span>🔄</span>
          <span>{{ scanning ? "扫描中..." : "扫描" }}</span>
        </button>
      </div>
    </header>

    <!-- 播放控制区 -->
    <div class="player-control-wrapper" v-if="playerStore.currentSong">
      <div class="player-info">
        <div class="player-cover">
          {{ playerStore.currentSong.title.charAt(0) }}
        </div>
        <div class="player-meta">
          <div class="player-song-title">{{ playerStore.currentSong.title }}</div>
          <div class="player-song-artist">{{ playerStore.currentSong.artist }}</div>
        </div>
      </div>
      
      <div class="player-controls">
        <button class="control-btn" @click="playerStore.previous" title="上一首">⏮</button>
        <button class="control-btn play-pause" @click="playerStore.togglePlay" :title="playerStore.isPlaying ? '暂停' : '播放'">
          {{ playerStore.isPlaying ? '⏸' : '▶' }}
        </button>
        <button class="control-btn" @click="playerStore.next" title="下一首">⏭</button>
      </div>

      <div class="player-progress">
        <span class="time-current">{{ formatDuration(playerStore.progress) }}</span>
        <input
          type="range"
          class="progress-bar"
          :value="playerStore.progress"
          :max="playerStore.duration"
          @input="(e) => playerStore.setProgress(Number((e.target as HTMLInputElement).value))"
        />
        <span class="time-total">{{ formatDuration(playerStore.duration) }}</span>
      </div>

      <div class="player-mode">
        <button class="mode-btn" @click="cycleMode" :title="modeLabel">
          <span class="mode-icon">{{ modeIcon }}</span>
          <span class="mode-text">{{ modeLabel }}</span>
        </button>
      </div>

      <div class="player-volume-outer">
        <span class="volume-icon">🔊</span>
        <input
          type="range"
          class="volume-bar"
          :value="playerStore.volume * 100"
          min="0"
          max="100"
          @input="(e) => playerStore.setVolume(Number((e.target as HTMLInputElement).value) / 100)"
        />
      </div>
    </div>

    <div v-if="errorMsg" class="error-msg">{{ errorMsg }}</div>

    <!-- 歌曲列表容器 -->
    <div class="song-list-container">
      <div class="song-list">
        <div
          v-for="song in displayedSongs"
          :key="song.id"
          :id="'song-' + song.id"
          class="song-item"
          :class="{
            playing: playerStore.currentSong?.id === song.id,
            selected: selectedSongIds.has(song.id),
          }"
          @click="handleSongClick($event, song)"
          @dblclick="playSong(song)"
          @contextmenu.prevent="openContextMenu($event, song)"
        >
          <div class="song-check">
            <div
              class="check-box"
              :class="{ checked: selectedSongIds.has(song.id) }"
              @click.stop="toggleSelect(song.id)"
            >
              {{ selectedSongIds.has(song.id) ? "✓" : "" }}
            </div>
          </div>
          <div class="song-title">
            <span v-if="playerStore.currentSong?.id === song.id" class="playing-icon">▶</span>
            {{ song.title }}
          </div>
          <div class="song-artist">{{ song.artist }}</div>
          <div class="song-album">{{ song.album }}</div>
          <div class="song-duration">{{ formatDuration(song.duration) }}</div>
        </div>
        <div v-if="displayedSongs.length === 0 && !scanning" class="empty-tip">
          {{ searchKeyword ? '未找到匹配的歌曲' : '暂无歌曲，点击「扫描文件夹」开始添加音乐' }}
        </div>
      </div>
    </div>

    <!-- 上下文菜单（含直接展示的歌单列表） -->
    <div
      v-if="contextMenu.visible"
      class="context-menu"
      :style="{ top: contextMenu.y + 'px', left: contextMenu.x + 'px' }"
    >
      <div class="context-menu-item" @click="playSong(contextMenu.song!)">播放</div>
      <div class="context-menu-divider"></div>
      <div class="context-menu-label">添加到歌单</div>
      <div
        v-for="pl in playlists"
        :key="pl.id"
        class="context-menu-item pl-item"
        @click="addToPlaylist(pl.id, pl.name)"
      >
        {{ pl.name }}
      </div>
      <div v-if="playlists.length === 0" class="context-menu-item disabled">
        暂无歌单
      </div>
    </div>

    <!-- Toast 提示 -->
    <div v-if="toast.visible" class="toast" :class="{ show: toast.visible }">
      {{ toast.message }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, nextTick, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { usePlayerStore } from "../stores/player";
import type { Song } from "../stores/player";
import { formatDuration } from "../utils/format";

const playerStore = usePlayerStore();
const songs = ref<Song[]>([]);
const scanning = ref(false);
const errorMsg = ref("");
const searchKeyword = ref("");
const searchTimer = ref<ReturnType<typeof setTimeout> | null>(null);

// 多选歌曲
const selectedSongIds = ref<Set<number>>(new Set());
const lastClickedId = ref<number | null>(null);

// 播放模式相关
const modeIcon = computed(() => {
  switch (playerStore.playMode) {
    case "loop_list":
      return "🔁";
    case "loop_single":
      return "🔂";
    case "shuffle":
      return "🔀";
    default:
      return "➡️";
  }
});

const modeLabel = computed(() => {
  switch (playerStore.playMode) {
    case "loop_list":
      return "列表";
    case "loop_single":
      return "单曲";
    case "shuffle":
      return "随机";
    default:
      return "顺序";
  }
});

function cycleMode() {
  const modes: Array<typeof playerStore.playMode> = [
    "sequence",
    "loop_list",
    "loop_single",
    "shuffle",
  ];
  const idx = modes.indexOf(playerStore.playMode);
  playerStore.setPlayMode(modes[(idx + 1) % modes.length]);
}

const displayedSongs = computed(() => {
  if (!searchKeyword.value.trim()) return songs.value;
  return songs.value; // 搜索时后端已过滤
});

// 上下文菜单
const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  song: null as Song | null,
});

// 歌单列表（预加载）
const playlists = ref<{ id: number; name: string }[]>([]);

// Toast
const toast = ref({ visible: false, message: "" });
let toastTimer: ReturnType<typeof setTimeout> | null = null;

function showToast(message: string) {
  if (toastTimer) clearTimeout(toastTimer);
  toast.value = { visible: true, message };
  toastTimer = setTimeout(() => {
    toast.value.visible = false;
  }, 2500);
}

async function fetchSongs() {
  try {
    const result = await invoke<Song[]>("get_all_songs");
    songs.value = result;
  } catch (e) {
    console.error("Failed to fetch songs:", e);
    errorMsg.value = "获取歌曲列表失败: " + String(e);
  }
}

async function fetchPlaylists() {
  try {
    const result = await invoke<{ id: number; name: string }[]>("get_playlists");
    playlists.value = result;
  } catch (e) {
    console.error("Failed to fetch playlists:", e);
  }
}

async function searchSongs() {
  if (!searchKeyword.value.trim()) {
    await fetchSongs();
    return;
  }
  try {
    const result = await invoke<Song[]>("search_songs", {
      keyword: searchKeyword.value.trim(),
    });
    songs.value = result;
  } catch (e) {
    console.error("Failed to search songs:", e);
    errorMsg.value = "搜索失败: " + String(e);
  }
}

function onSearch() {
  if (searchTimer.value) clearTimeout(searchTimer.value);
  searchTimer.value = setTimeout(() => {
    searchSongs();
  }, 300);
}

async function scanLibrary() {
  scanning.value = true;
  errorMsg.value = "";
  try {
    const count = await invoke<number>("scan_library");
    console.log(`Scanned ${count} songs`);
    if (count === 0) {
      errorMsg.value = "未扫描到歌曲。请先在「设置」中添加音乐库文件夹。";
    }
    await fetchSongs();
  } catch (e) {
    console.error("Failed to scan library:", e);
    errorMsg.value = "扫描失败: " + String(e);
  } finally {
    scanning.value = false;
  }
}

function playSong(song: Song) {
  closeContextMenu();
  playerStore.playSong(song, songs.value);
}

// 播放全部
function playAll() {
  if (displayedSongs.value.length > 0) {
    playerStore.playSong(displayedSongs.value[0], displayedSongs.value);
  }
}

// 随机播放
function shufflePlay() {
  if (displayedSongs.value.length > 0) {
    const shuffled = [...displayedSongs.value].sort(() => Math.random() - 0.5);
    playerStore.playSong(shuffled[0], shuffled);
  }
}

function toggleSelect(songId: number) {
  if (selectedSongIds.value.has(songId)) {
    selectedSongIds.value.delete(songId);
  } else {
    selectedSongIds.value.add(songId);
  }
  lastClickedId.value = songId;
}

function handleSongClick(e: MouseEvent, song: Song) {
  if (e.ctrlKey || e.metaKey) {
    toggleSelect(song.id);
  } else if (e.shiftKey && lastClickedId.value !== null) {
    const ids = songs.value.map((s) => s.id);
    const start = ids.indexOf(lastClickedId.value);
    const end = ids.indexOf(song.id);
    if (start !== -1 && end !== -1) {
      const min = Math.min(start, end);
      const max = Math.max(start, end);
      for (let i = min; i <= max; i++) {
        selectedSongIds.value.add(ids[i]);
      }
    }
  } else {
    selectedSongIds.value.clear();
    selectedSongIds.value.add(song.id);
    lastClickedId.value = song.id;
  }
}

function openContextMenu(e: MouseEvent, song: Song) {
  // 如果右键的歌曲不在已选列表中，则单独选中它
  if (!selectedSongIds.value.has(song.id)) {
    selectedSongIds.value.clear();
    selectedSongIds.value.add(song.id);
    lastClickedId.value = song.id;
  }
  contextMenu.value = {
    visible: true,
    x: e.clientX,
    y: e.clientY,
    song,
  };
  // 歌单已预加载，无需额外请求
}

async function addToPlaylist(playlistId: number, playlistName: string) {
  const ids = Array.from(selectedSongIds.value);
  if (ids.length === 0) return;

  let successCount = 0;
  let failCount = 0;

  for (const songId of ids) {
    try {
      await invoke("add_song_to_playlist", {
        playlistId: playlistId,
        songId: songId,
      });
      successCount++;
    } catch (e) {
      console.error("Failed to add song to playlist:", e);
      failCount++;
    }
  }

  closeContextMenu();

  if (successCount > 0) {
    showToast(`已添加 ${successCount} 首歌曲到歌单「${playlistName}」`);
  }
  if (failCount > 0) {
    showToast(`${failCount} 首歌曲添加失败`);
  }
}

function closeContextMenu() {
  contextMenu.value.visible = false;
}

// 点击空白处关闭菜单
document.addEventListener("click", (e) => {
  const target = e.target as HTMLElement;
  if (!target.closest(".context-menu")) {
    closeContextMenu();
  }
});

watch(
  () => playerStore.currentSong,
  (newSong) => {
    if (newSong) {
      nextTick(() => {
        const el = document.getElementById("song-" + newSong.id);
        if (el) {
          const rect = el.getBoundingClientRect();
          const container = document.querySelector(".library-view");
          if (container) {
            const containerRect = container.getBoundingClientRect();
            const isVisible =
              rect.top >= containerRect.top && rect.bottom <= containerRect.bottom;
            if (!isVisible) {
              el.scrollIntoView({ behavior: "smooth", block: "center" });
            }
          }
        }
      });
    }
  }
);

onMounted(() => {
  fetchSongs();
  fetchPlaylists(); // 预加载歌单
});
</script>

<style scoped>
.library-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
}

/* 固定头部区域 */
.library-view > :not(.song-list-container):not(.context-menu):not(.toast) {
  flex-shrink: 0;
  margin: 0 24px;
}

.library-view > .library-header-compact {
  margin-top: 24px;
}

.library-view > .player-control-wrapper {
  margin-top: 20px;
}

.library-view > .error-msg {
  margin-top: 16px;
  padding: 12px 16px;
}

/* 音乐库横幅 */
.library-header-compact {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.08) 0%, rgba(30, 41, 59, 0.4) 100%);
  border-radius: 10px;
  gap: 20px;
  flex-wrap: wrap;
}

/* 播放控制区 */
.player-control-wrapper {
  display: flex;
  align-items: center;
  gap: 16px;
}

/* 播放控制区主体（蓝色边框内） */
.player-control-section {
  display: grid;
  grid-template-columns: minmax(140px, 180px) auto minmax(200px, 1fr) minmax(90px, 110px);
  grid-template-areas: "info controls progress mode";
  align-items: center;
  gap: 16px;
  padding: 14px 18px;
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.08) 0%, rgba(30, 41, 59, 0.4) 100%);
  border-radius: 10px;
  border: 1px solid rgba(59, 130, 246, 0.2);
  flex: 1;
  min-width: 0;
}

/* 音量控制（蓝色边框外） */
.player-volume-outer {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 120px;
}

/* 歌曲列表容器 */
.song-list-container {
  flex: 1;
  margin: 24px 24px 24px 24px;
  background: rgba(30, 41, 59, 0.3);
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.05);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.song-list {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 14px;
  flex-shrink: 0;
}

.library-icon-small {
  width: 50px;
  height: 50px;
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  flex-shrink: 0;
}

.library-title-compact {
  font-size: 24px;
  font-weight: 700;
  margin: 0;
  color: #f8fafc;
  line-height: 1.2;
}

.library-subtitle {
  font-size: 12px;
  color: #94a3b8;
  margin: 2px 0 0 0;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
  justify-content: flex-end;
  min-width: 0;
  flex-wrap: wrap;
}

.search-box-inline {
  display: flex;
  align-items: center;
  gap: 10px;
  background-color: #1e293b;
  border: 1px solid #334155;
  border-radius: 8px;
  padding: 10px 14px;
  transition: border-color 0.2s;
  flex: 1 1 200px;
  min-width: 150px;
  max-width: 400px;
}

.search-box-inline:focus-within {
  border-color: #3b82f6;
}

.search-icon {
  font-size: 14px;
  color: #64748b;
  flex-shrink: 0;
}

.search-input-inline {
  flex: 1;
  background: none;
  border: none;
  color: #e2e8f0;
  font-size: 15px;
  outline: none;
  min-width: 0;
}

.search-input-inline::placeholder {
  color: #64748b;
}

.action-btn-compact {
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
  flex-shrink: 0;
}

/* 响应式调整 */
@media (max-width: 1200px) {
  .header-right {
    gap: 8px;
  }
  
  .search-box-inline {
    flex: 1 1 150px;
    min-width: 120px;
    padding: 8px 12px;
  }
  
  .action-btn-compact {
    padding: 8px 16px;
    font-size: 13px;
    gap: 4px;
  }
  
  .action-btn-compact span:last-child {
    display: none;
  }
  
  .action-btn-compact span:first-child {
    margin: 0;
  }
  
  /* 播放控制区响应式 */
  .player-control-wrapper {
    grid-template-columns: minmax(120px, 160px) auto minmax(180px, 1fr) minmax(80px, 100px) minmax(90px, 110px);
    gap: 12px;
    padding: 12px 16px;
  }
  
  .player-cover {
    width: 40px;
    height: 40px;
    font-size: 18px;
  }
  
  .player-song-title {
    font-size: 13px;
  }
  
  .player-song-artist {
    font-size: 11px;
  }
  
  .control-btn {
    width: 32px;
    height: 32px;
    font-size: 12px;
  }
  
  .control-btn.play-pause {
    width: 40px;
    height: 40px;
    font-size: 14px;
  }
  
  .mode-btn {
    padding: 5px 10px;
  }
  
  .mode-text {
    font-size: 10px;
  }
}

@media (max-width: 900px) {
  .library-header-compact {
    gap: 12px;
  }
  
  .header-left {
    gap: 10px;
  }
  
  .library-icon-small {
    width: 40px;
    height: 40px;
    font-size: 20px;
  }
  
  .library-title-compact {
    font-size: 20px;
  }
  
  .library-subtitle {
    font-size: 11px;
  }
  
  .search-box-inline {
    flex: 1 1 100px;
    min-width: 100px;
  }
  
  .action-btn-compact {
    padding: 8px 12px;
    font-size: 12px;
  }
  
  /* 播放控制区垂直布局 */
  .player-control-wrapper {
    grid-template-columns: 1fr;
    grid-template-areas:
      "info"
      "controls"
      "progress"
      "extras";
    gap: 12px;
  }
  
  .player-info {
    justify-content: center;
  }
  
  .player-controls {
    justify-content: center;
  }
  
  .player-mode,
  .player-volume-outer {
    grid-area: extras;
    justify-content: center;
  }
  
  .player-mode {
    display: none;
  }
}

@media (max-width: 700px) {
  .header-right {
    width: 100%;
    justify-content: stretch;
  }
  
  .search-box-inline {
    flex: 1;
    max-width: none;
  }
  
  .action-btn-compact {
    flex: 1;
  }
  
  .player-control-wrapper {
    padding: 10px 12px;
  }
  
  .time-current,
  .time-total {
    font-size: 11px;
    min-width: 35px;
  }
  
  .player-volume-outer .volume-icon {
    font-size: 14px;
  }
}

.action-btn-compact.play-all {
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  color: white;
}

.action-btn-compact.play-all:hover {
  background: linear-gradient(135deg, #2563eb 0%, #1d4ed8 100%);
  transform: translateY(-1px);
}

.action-btn-compact.shuffle {
  background: rgba(255, 255, 255, 0.08);
  color: #e2e8f0;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.action-btn-compact.shuffle:hover {
  background: rgba(255, 255, 255, 0.12);
}

.action-btn-compact.scan {
  background-color: rgba(16, 185, 129, 0.15);
  color: #34d399;
}

.action-btn-compact.scan:hover:not(:disabled) {
  background-color: rgba(16, 185, 129, 0.25);
}

.action-btn-compact.scan:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.error-msg {
  background-color: rgba(239, 68, 68, 0.15);
  color: #fca5a5;
  border-radius: 6px;
  font-size: 14px;
}

.song-item {
  display: grid;
  grid-template-columns: 36px 2fr 1fr 1fr 80px;
  padding: 10px 14px;
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 0.15s;
  user-select: none;
  -webkit-user-select: none;
  align-items: center;
}

.song-item:hover {
  background-color: rgba(255, 255, 255, 0.05);
}

.song-item.selected {
  background-color: rgba(59, 130, 246, 0.15);
}

.song-item.playing {
  background-color: rgba(59, 130, 246, 0.15);
  border-left: 3px solid #3b82f6;
}

.song-item.playing .song-title {
  color: #3b82f6;
  font-weight: 600;
}

.song-check {
  display: flex;
  align-items: center;
  justify-content: center;
}

.check-box {
  width: 16px;
  height: 16px;
  border: 1.5px solid #64748b;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  color: white;
  cursor: pointer;
  transition: all 0.15s;
}

.check-box.checked {
  background-color: #3b82f6;
  border-color: #3b82f6;
}

.playing-icon {
  color: #3b82f6;
  font-size: 10px;
  margin-right: 8px;
}

.song-title {
  font-weight: 500;
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.song-artist,
.song-album {
  color: #94a3b8;
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.song-duration {
  color: #94a3b8;
  font-size: 14px;
  text-align: right;
}

.empty-tip {
  padding: 40px;
  text-align: center;
  color: #64748b;
  font-size: 14px;
}

/* 播放控制区元素 */
.player-info {
  grid-area: info;
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.player-cover {
  width: 48px;
  height: 48px;
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  font-weight: 700;
  color: white;
  flex-shrink: 0;
}

.player-meta {
  flex: 1;
  min-width: 0;
}

.player-song-title {
  font-size: 14px;
  font-weight: 600;
  color: #e2e8f0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.player-song-artist {
  font-size: 12px;
  color: #94a3b8;
  margin-top: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.player-controls {
  grid-area: controls;
  display: flex;
  align-items: center;
  gap: 8px;
}

.control-btn {
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.08);
  color: #e2e8f0;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.control-btn:hover {
  background: rgba(255, 255, 255, 0.15);
  transform: scale(1.05);
}

.control-btn.play-pause {
  width: 44px;
  height: 44px;
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  font-size: 16px;
}

.control-btn.play-pause:hover {
  background: linear-gradient(135deg, #2563eb 0%, #1d4ed8 100%);
}

.player-progress {
  grid-area: progress;
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.time-current,
.time-total {
  font-size: 12px;
  color: #94a3b8;
  min-width: 40px;
  text-align: center;
  flex-shrink: 0;
}

.progress-bar {
  flex: 1;
  height: 4px;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.1);
  outline: none;
  cursor: pointer;
  -webkit-appearance: none;
}

.progress-bar::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #3b82f6;
  cursor: pointer;
  transition: all 0.2s;
}

.progress-bar::-webkit-slider-thumb:hover {
  background: #2563eb;
  transform: scale(1.2);
}

.player-mode {
  grid-area: mode;
  display: flex;
  align-items: center;
  justify-content: center;
}

.mode-btn {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  padding: 6px 12px;
  color: #e2e8f0;
  font-size: 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 6px;
  transition: all 0.2s;
  white-space: nowrap;
}

.mode-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  border-color: rgba(255, 255, 255, 0.2);
}

.mode-icon {
  font-size: 14px;
}

.mode-text {
  font-size: 11px;
  font-weight: 500;
}

.player-volume-outer {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 120px;
}

.volume-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.volume-bar {
  flex: 1;
  height: 4px;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.1);
  outline: none;
  cursor: pointer;
  -webkit-appearance: none;
}

.volume-bar::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #10b981;
  cursor: pointer;
  transition: all 0.2s;
}

.volume-bar::-webkit-slider-thumb:hover {
  background: #059669;
  transform: scale(1.2);
}

/* 播放控制区响应式 */
@media (max-width: 1200px) {
  .player-control-wrapper {
    gap: 12px;
  }
  
  .player-control-section {
    grid-template-columns: minmax(120px, 160px) auto 1fr minmax(80px, 100px);
    gap: 12px;
    padding: 12px 16px;
  }
  
  .player-volume-outer {
    min-width: 90px;
  }
  
  .player-cover {
    width: 40px;
    height: 40px;
    font-size: 18px;
  }
  
  .player-song-title {
    font-size: 13px;
  }
  
  .player-song-artist {
    font-size: 11px;
  }
  
  .control-btn {
    width: 32px;
    height: 32px;
    font-size: 12px;
  }
  
  .control-btn.play-pause {
    width: 40px;
    height: 40px;
    font-size: 14px;
  }
  
  .mode-btn {
    padding: 5px 10px;
  }
  
  .mode-text {
    font-size: 10px;
  }
}

@media (max-width: 900px) {
  .player-control-wrapper {
    flex-direction: column;
    gap: 12px;
  }
  
  .player-control-section {
    grid-template-columns: 1fr;
    grid-template-areas:
      "info"
      "controls"
      "progress";
    gap: 12px;
    width: 100%;
  }
  
  .player-info {
    justify-content: center;
  }
  
  .player-controls {
    justify-content: center;
  }
  
  .player-mode {
    display: none;
  }
  
  .player-volume-outer {
    justify-content: center;
    width: 100%;
    max-width: 300px;
    margin: 0 auto;
  }
}

@media (max-width: 700px) {
  .player-control-section {
    padding: 10px 12px;
  }
  
  .time-current,
  .time-total {
    font-size: 11px;
    min-width: 35px;
  }
  
  .player-volume-outer .volume-icon {
    font-size: 14px;
  }
}

/* 上下文菜单 */
.context-menu {
  position: fixed;
  background-color: #1e293b;
  border: 1px solid #334155;
  border-radius: 8px;
  padding: 6px 0;
  min-width: 180px;
  max-height: 320px;
  overflow-y: auto;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  z-index: 1000;
}

.context-menu-item {
  padding: 8px 16px;
  font-size: 14px;
  color: #e2e8f0;
  cursor: pointer;
  transition: background-color 0.15s;
}

.context-menu-item:hover {
  background-color: rgba(59, 130, 246, 0.2);
}

.context-menu-item.disabled {
  color: #64748b;
  cursor: default;
}

.context-menu-item.disabled:hover {
  background-color: transparent;
}

.context-menu-item.pl-item {
  padding-left: 24px;
  font-size: 13px;
}

.context-menu-divider {
  height: 1px;
  background-color: rgba(255, 255, 255, 0.08);
  margin: 4px 0;
}

.context-menu-label {
  padding: 6px 16px;
  font-size: 12px;
  color: #64748b;
  cursor: default;
}

/* Toast */
.toast {
  position: fixed;
  bottom: 80px;
  left: 50%;
  transform: translateX(-50%) translateY(20px);
  background-color: #1e293b;
  color: #e2e8f0;
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 14px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  border: 1px solid #334155;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.3s, transform 0.3s;
  z-index: 2000;
}

.toast.show {
  opacity: 1;
  transform: translateX(-50%) translateY(0);
}
</style>


.action-btn-compact.play-all {
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  color: white;
}

.action-btn-compact.play-all:hover {
  background: linear-gradient(135deg, #2563eb 0%, #1d4ed8 100%);
  transform: translateY(-1px);
}

.action-btn-compact.shuffle {
  background: rgba(255, 255, 255, 0.08);
  color: #e2e8f0;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.action-btn-compact.shuffle:hover {
  background: rgba(255, 255, 255, 0.12);
}

.action-btn-compact.scan {
  background-color: rgba(16, 185, 129, 0.15);
  color: #34d399;
}

.action-btn-compact.scan:hover:not(:disabled) {
  background-color: rgba(16, 185, 129, 0.25);
}

.action-btn-compact.scan:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.empty-tip {
  padding: 40px;
  text-align: center;
  color: #64748b;
  font-size: 14px;
}

/* 上下文菜单 */
.context-menu {
  position: fixed;
  background-color: #1e293b;
  border: 1px solid #334155;
  border-radius: 8px;
  padding: 6px 0;
  min-width: 180px;
  max-height: 320px;
  overflow-y: auto;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  z-index: 1000;
}

.context-menu-item {
  padding: 8px 16px;
  font-size: 14px;
  color: #e2e8f0;
  cursor: pointer;
  transition: background-color 0.15s;
}

.context-menu-item:hover {
  background-color: rgba(59, 130, 246, 0.2);
}

.context-menu-item.disabled {
  color: #64748b;
  cursor: default;
}

.context-menu-item.disabled:hover {
  background-color: transparent;
}

.context-menu-item.pl-item {
  padding-left: 24px;
  font-size: 13px;
}

.context-menu-divider {
  height: 1px;
  background-color: rgba(255, 255, 255, 0.08);
  margin: 4px 0;
}

.context-menu-label {
  padding: 6px 16px;
  font-size: 12px;
  color: #64748b;
  cursor: default;
}

/* Toast */
.toast {
  position: fixed;
  bottom: 80px;
  left: 50%;
  transform: translateX(-50%) translateY(20px);
  background-color: #1e293b;
  color: #e2e8f0;
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 14px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  border: 1px solid #334155;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.3s, transform 0.3s;
  z-index: 2000;
}

.toast.show {
  opacity: 1;
  transform: translateX(-50%) translateY(0);
}
