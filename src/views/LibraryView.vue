<template>
  <div class="library-view">
    <header class="library-header">
      <h1>音乐库</h1>
      <div class="header-actions">
        <input
          v-model="searchKeyword"
          type="text"
          class="search-input"
          placeholder="搜索歌曲、艺术家、专辑..."
          @input="onSearch"
        />
        <button class="scan-btn" @click="scanLibrary" :disabled="scanning">
          {{ scanning ? "扫描中..." : "扫描文件夹" }}
        </button>
      </div>
    </header>
    <div v-if="errorMsg" class="error-msg">{{ errorMsg }}</div>
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
const pendingSongIds = ref<number[]>([]);

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
  padding: 24px;
  overflow-y: auto;
  position: relative;
}

.library-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  gap: 16px;
}

.library-header h1 {
  font-size: 24px;
  font-weight: 600;
  white-space: nowrap;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
  justify-content: flex-end;
}

.search-input {
  padding: 8px 14px;
  background-color: #1e293b;
  border: 1px solid #334155;
  border-radius: 6px;
  color: #e2e8f0;
  font-size: 14px;
  width: 260px;
  outline: none;
  transition: border-color 0.2s;
}

.search-input:focus {
  border-color: #3b82f6;
}

.search-input::placeholder {
  color: #64748b;
}

.scan-btn {
  padding: 8px 16px;
  background-color: #3b82f6;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  white-space: nowrap;
}

.scan-btn:hover:not(:disabled) {
  background-color: #2563eb;
}

.scan-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.error-msg {
  padding: 12px 16px;
  background-color: rgba(239, 68, 68, 0.15);
  color: #fca5a5;
  border-radius: 6px;
  margin-bottom: 16px;
  font-size: 14px;
}

.song-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
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
