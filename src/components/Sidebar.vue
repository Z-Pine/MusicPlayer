<template>
  <aside class="sidebar">
    <div class="logo">
      <span class="logo-icon">🎵</span>
      <span class="logo-text">逐浪音乐</span>
    </div>
    
    <nav class="nav-menu">
      <router-link to="/" class="nav-item" active-class="active">
        <span class="nav-icon">🏠</span>
        <span>音乐库</span>
      </router-link>
      <router-link to="/playlist/recent" class="nav-item" active-class="active">
        <span class="nav-icon">🕐</span>
        <span>最近播放</span>
      </router-link>
      <router-link to="/settings" class="nav-item" active-class="active">
        <span class="nav-icon">⚙️</span>
        <span>设置</span>
      </router-link>
    </nav>

    <div class="playlists-section">
      <div class="section-header">
        <span>我的歌单</span>
        <button class="add-btn" @click="createPlaylist">+</button>
      </div>
      <div class="playlist-list">
        <div
          v-for="pl in playlists"
          :key="pl.id"
          class="playlist-item"
          :class="{ active: isPlaylistActive(pl.id) }"
          @click="openPlaylist(pl)"
          @contextmenu.prevent="openPlaylistContextMenu($event, pl)"
        >
          📁 {{ pl.name }}
        </div>
      </div>
    </div>

    <!-- 歌单右键菜单 -->
    <div
      v-if="contextMenu.visible"
      class="context-menu"
      :style="{ top: contextMenu.y + 'px', left: contextMenu.x + 'px' }"
    >
      <div class="context-menu-item" @click="addSongsToPlaylist">添加歌曲</div>
      <div class="context-menu-divider"></div>
      <div class="context-menu-item" @click="renamePlaylist">重命名</div>
      <div class="context-menu-divider"></div>
      <div class="context-menu-item delete" @click="deletePlaylist">删除歌单</div>
    </div>

    <!-- 添加歌曲弹窗 -->
    <div v-if="addModalOpen" class="modal-overlay" @click="closeAddSongModal">
      <div class="modal-content" @click.stop>
        <h3 class="modal-title">添加歌曲到「{{ addModalPlaylistName }}」</h3>
        <div class="modal-search">
          <input
            v-model="addModalSearch"
            type="text"
            class="modal-search-input"
            placeholder="搜索歌曲..."
            @input="onAddModalSearch"
            ref="addModalSearchInput"
          />
        </div>
        <div class="modal-song-list">
          <div v-if="addModalLoading" class="modal-empty">加载中...</div>
          <div v-else-if="addModalError" class="modal-empty" style="color: #ef4444;">{{ addModalError }}</div>
          <template v-else>
            <div
              v-for="song in addModalSongs"
              :key="song.id"
              class="modal-song-item"
              :class="{ selected: addModalSelected.includes(song.id) }"
              @click="toggleSelectSong(song.id)"
            >
              <div class="check-box">
                {{ addModalSelected.includes(song.id) ? '✓' : '' }}
              </div>
              <div class="modal-song-title">{{ song.title }}</div>
              <div class="modal-song-artist">{{ song.artist }}</div>
            </div>
            <div v-if="addModalSongs.length === 0" class="modal-empty">
              暂无歌曲
            </div>
          </template>
        </div>
        <div class="modal-actions">
          <button class="modal-btn cancel" @click="closeAddSongModal">取消</button>
          <button class="modal-btn confirm" @click="confirmAddSongs">
            添加 ({{ addModalSelected.length }})
          </button>
        </div>
      </div>
    </div>

    <!-- 音乐库统计卡片 -->
    <div class="stats-card" v-if="libraryStats">
      <div class="card-header">📊 音乐库统计</div>
      <div class="stats-grid">
        <div class="stat-item">
          <div class="stat-value">{{ libraryStats.totalSongs }}</div>
          <div class="stat-label">首歌曲</div>
        </div>
        <div class="stat-item">
          <div class="stat-value">{{ libraryStats.totalArtists }}</div>
          <div class="stat-label">位艺术家</div>
        </div>
        <div class="stat-item">
          <div class="stat-value">{{ libraryStats.totalAlbums }}</div>
          <div class="stat-label">张专辑</div>
        </div>
        <div class="stat-item">
          <div class="stat-value">{{ libraryStats.totalDuration }}</div>
          <div class="stat-label">总时长</div>
        </div>
      </div>
    </div>

    <!-- 正在播放卡片 -->
    <div class="now-playing-card" v-if="playerStore.currentSong">
      <div class="card-header">🎵 正在播放</div>
      <div class="now-playing-content">
        <div class="now-playing-cover">
          {{ playerStore.currentSong.title.charAt(0) }}
        </div>
        <div class="now-playing-info">
          <div class="now-playing-title">{{ playerStore.currentSong.title }}</div>
          <div class="now-playing-artist">{{ playerStore.currentSong.artist }}</div>
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { useRouter, useRoute } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { usePlayerStore } from "../stores/player";
import { useDialog } from "../composables/useDialog";
import type { PlaylistItem, Song } from "../stores/player";

const router = useRouter();
const route = useRoute();
const playerStore = usePlayerStore();
const dialog = useDialog();
const playlists = ref<PlaylistItem[]>([]);
const allSongs = ref<Song[]>([]);

// 右键菜单
const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  playlist: null as PlaylistItem | null,
});

// 添加歌曲弹窗相关
const addModalOpen = ref(false);
const addModalSearch = ref("");
const addModalSongs = ref<Song[]>([]);
const addModalSelected = ref<number[]>([]);
const addModalSearchTimer = ref<ReturnType<typeof setTimeout> | null>(null);
const addModalExistingSongIds = ref<Set<number>>(new Set());
const addModalLoading = ref(false);
const addModalError = ref("");
const addModalPlaylistId = ref<number | null>(null);
const addModalPlaylistName = ref("");
const addModalSearchInput = ref<HTMLInputElement | null>(null);

// 计算音乐库统计信息
const libraryStats = computed(() => {
  if (allSongs.value.length === 0) return null;
  
  const artists = new Set(allSongs.value.map(s => s.artist));
  const albums = new Set(allSongs.value.map(s => s.album));
  const totalSeconds = allSongs.value.reduce((sum, song) => sum + song.duration, 0);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  
  return {
    totalSongs: allSongs.value.length,
    totalArtists: artists.size,
    totalAlbums: albums.size,
    totalDuration: hours > 0 ? `${hours}h ${minutes}m` : `${minutes}m`
  };
});

async function fetchPlaylists() {
  try {
    const result = await invoke<PlaylistItem[]>("get_playlists");
    playlists.value = result;
  } catch (e) {
    console.error("Failed to fetch playlists:", e);
  }
}

async function fetchAllSongs() {
  try {
    const result = await invoke<Song[]>("get_all_songs_basic");
    allSongs.value = result;
  } catch (e) {
    console.error("Failed to fetch songs:", e);
  }
}

function onPlaylistsUpdated() {
  fetchPlaylists();
}

async function createPlaylist() {
  const name = await dialog.prompt('', '创建歌单', '', '请输入歌单名称');
  if (name && name.trim()) {
    // 检查是否有重名的歌单
    const existingPlaylist = playlists.value.find(pl => pl.name === name.trim());
    if (existingPlaylist) {
      await dialog.error(`歌单「${name.trim()}」已存在，请使用其他名称`);
      return;
    }
    
    try {
      await invoke("create_playlist", { name: name.trim() });
      await fetchPlaylists();
    } catch (e) {
      console.error("Failed to create playlist:", e);
      await dialog.error("创建歌单失败: " + String(e));
    }
  }
}

function openPlaylist(pl: PlaylistItem) {
  router.push(`/playlist/${pl.id}`);
}

function isPlaylistActive(playlistId: number): boolean {
  return route.name === "playlist" && route.params.id === String(playlistId);
}

function openPlaylistContextMenu(e: MouseEvent, pl: PlaylistItem) {
  contextMenu.value = {
    visible: true,
    x: e.clientX,
    y: e.clientY,
    playlist: pl,
  };
}

function closeContextMenu() {
  contextMenu.value.visible = false;
}

async function addSongsToPlaylist() {
  const pl = contextMenu.value.playlist;
  closeContextMenu();
  if (!pl) return;
  
  // 打开添加歌曲弹窗
  addModalPlaylistId.value = pl.id;
  addModalPlaylistName.value = pl.name;
  addModalOpen.value = true;
  addModalSearch.value = "";
  addModalSelected.value = [];
  addModalSongs.value = [];
  
  // 获取歌单中已有的歌曲ID，用于去重
  try {
    const result = await invoke<{
      playlist_id: number;
      song_id: number;
      position: number;
      song: Song | null;
    }[]>("get_playlist_songs", { playlistId: pl.id });
    addModalExistingSongIds.value = new Set(
      result.map((r) => r.song?.id).filter((id): id is number => id !== null && id !== undefined)
    );
  } catch (e) {
    console.error("Failed to get playlist songs:", e);
    addModalExistingSongIds.value = new Set();
  }
  
  // 自动加载全部歌曲
  await loadAddModalSongs();
  
  // 自动聚焦到搜索框
  setTimeout(() => {
    addModalSearchInput.value?.focus();
  }, 100);
  
  // 添加键盘事件监听
  document.addEventListener('keydown', handleAddModalKeydown);
}

function closeAddSongModal() {
  addModalOpen.value = false;
  addModalSearch.value = "";
  addModalSelected.value = [];
  addModalSongs.value = [];
  addModalPlaylistId.value = null;
  addModalPlaylistName.value = "";
  // 移除键盘事件监听
  document.removeEventListener('keydown', handleAddModalKeydown);
}

function handleAddModalKeydown(e: KeyboardEvent) {
  if (!addModalOpen.value) return;
  
  if (e.key === 'Escape') {
    e.preventDefault();
    closeAddSongModal();
  } else if (e.key === 'Enter' && !e.shiftKey && !e.ctrlKey && !e.altKey && !e.metaKey) {
    // 只有在不是输入框中按 Enter 时才触发确认
    const target = e.target as HTMLElement;
    if (target.tagName !== 'INPUT' && target.tagName !== 'TEXTAREA') {
      e.preventDefault();
      confirmAddSongs();
    }
  }
}

async function loadAddModalSongs() {
  addModalLoading.value = true;
  addModalError.value = "";
  try {
    if (addModalSearch.value.trim()) {
      const result = await invoke<Song[]>("search_songs", {
        keyword: addModalSearch.value.trim(),
      });
      addModalSongs.value = result.filter(
        (s) => !addModalExistingSongIds.value.has(s.id)
      );
    } else {
      const result = await invoke<Song[]>("get_all_songs_basic");
      addModalSongs.value = result.filter(
        (s) => !addModalExistingSongIds.value.has(s.id)
      );
    }
  } catch (e) {
    console.error("Failed to load songs for modal:", e);
    addModalError.value = "加载歌曲失败: " + String(e);
    addModalSongs.value = [];
  } finally {
    addModalLoading.value = false;
  }
}

function onAddModalSearch() {
  if (addModalSearchTimer.value) clearTimeout(addModalSearchTimer.value);
  addModalSearchTimer.value = setTimeout(() => {
    loadAddModalSongs();
  }, 300);
}

function toggleSelectSong(songId: number) {
  const idx = addModalSelected.value.indexOf(songId);
  if (idx >= 0) {
    addModalSelected.value.splice(idx, 1);
  } else {
    addModalSelected.value.push(songId);
  }
}

async function confirmAddSongs() {
  if (addModalPlaylistId.value === null || addModalSelected.value.length === 0) return;
  try {
    for (const songId of addModalSelected.value) {
      await invoke("add_song_to_playlist", {
        playlistId: addModalPlaylistId.value,
        songId: songId,
      });
    }
    closeAddSongModal();
    // 通知其他组件刷新
    window.dispatchEvent(new CustomEvent("playlists-updated"));
  } catch (e) {
    console.error("Failed to add songs:", e);
    await dialog.error("添加失败: " + String(e));
  }
}

async function renamePlaylist() {
  const pl = contextMenu.value.playlist;
  closeContextMenu();
  if (!pl) return;
  
  const newName = await dialog.prompt('', '重命名歌单', pl.name, '请输入新歌单名称');
  if (newName && newName.trim()) {
    try {
      await invoke("rename_playlist", {
        id: pl.id,
        name: newName.trim(),
      });
      await fetchPlaylists();
      // 通知其他组件刷新
      window.dispatchEvent(new CustomEvent("playlists-updated"));
    } catch (e) {
      console.error("Failed to rename playlist:", e);
      await dialog.error("重命名失败: " + String(e));
    }
  }
}

async function deletePlaylist() {
  const pl = contextMenu.value.playlist;
  closeContextMenu();
  if (!pl) return;
  
  const confirmed = await dialog.confirm({
    title: '确认删除',
    message: `确定要删除歌单「${pl.name}」吗？`,
    type: 'warning',
    confirmText: '删除',
    cancelText: '取消',
  });
  
  if (!confirmed) return;
  
  try {
    await invoke("delete_playlist", { id: pl.id });
    await fetchPlaylists();
    // 通知其他组件刷新
    window.dispatchEvent(new CustomEvent("playlists-updated"));
    // 如果当前正在查看被删除的歌单，跳转到音乐库
    if (isPlaylistActive(pl.id)) {
      router.push("/");
    }
  } catch (e) {
    console.error("Failed to delete playlist:", e);
    await dialog.error("删除失败: " + String(e));
  }
}

// 点击空白处关闭菜单
document.addEventListener("click", (e) => {
  const target = e.target as HTMLElement;
  if (!target.closest(".context-menu")) {
    closeContextMenu();
  }
});

onMounted(() => {
  fetchPlaylists();
  fetchAllSongs();
  window.addEventListener("playlists-updated", onPlaylistsUpdated);
});

onUnmounted(() => {
  window.removeEventListener("playlists-updated", onPlaylistsUpdated);
});
</script>

<style scoped>
.sidebar {
  width: 300px;
  background-color: #0f172a;
  border-right: 1px solid #1e293b;
  display: flex;
  flex-direction: column;
  padding: 20px 16px;
  gap: 16px;
  overflow-y: auto;
}

.logo {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 8px;
}

.logo-icon {
  font-size: 28px;
}

.logo-text {
  font-size: 20px;
  font-weight: 700;
  color: #f8fafc;
}

.nav-menu {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 8px;
  color: #94a3b8;
  text-decoration: none;
  font-size: 14px;
  transition: all 0.2s;
}

.nav-item:hover {
  background-color: rgba(255, 255, 255, 0.05);
  color: #e2e8f0;
}

.nav-item.active {
  background-color: rgba(59, 130, 246, 0.15);
  color: #3b82f6;
}

.nav-icon {
  font-size: 18px;
}

.playlists-section {
  flex: 1;
  overflow-y: auto;
  min-height: 120px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 8px;
  margin-bottom: 8px;
  font-size: 12px;
  font-weight: 600;
  color: #64748b;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.add-btn {
  background: none;
  border: none;
  color: #94a3b8;
  font-size: 20px;
  cursor: pointer;
  padding: 0 4px;
  transition: color 0.2s;
}

.add-btn:hover {
  color: #e2e8f0;
}

.playlist-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.playlist-item {
  padding: 8px 12px;
  border-radius: 6px;
  color: #94a3b8;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.playlist-item:hover {
  background-color: rgba(255, 255, 255, 0.05);
  color: #e2e8f0;
}

.playlist-item.active {
  background-color: rgba(59, 130, 246, 0.15);
  color: #3b82f6;
}

/* 统计卡片 */
.stats-card {
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.1) 0%, rgba(30, 41, 59, 0.5) 100%);
  border-radius: 10px;
  padding: 14px;
  border: 1px solid rgba(16, 185, 129, 0.2);
}

.card-header {
  font-size: 13px;
  font-weight: 600;
  color: #cbd5e1;
  margin-bottom: 12px;
}

.stats-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.stat-item {
  text-align: center;
}

.stat-value {
  font-size: 20px;
  font-weight: 700;
  color: #10b981;
  line-height: 1.2;
}

.stat-label {
  font-size: 11px;
  color: #94a3b8;
  margin-top: 2px;
}

/* 正在播放卡片 */
.now-playing-card {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.1) 0%, rgba(30, 41, 59, 0.5) 100%);
  border-radius: 10px;
  padding: 14px;
  border: 1px solid rgba(59, 130, 246, 0.2);
}

.now-playing-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.now-playing-cover {
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

.now-playing-info {
  flex: 1;
  min-width: 0;
}

.now-playing-title {
  font-size: 13px;
  font-weight: 600;
  color: #e2e8f0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.now-playing-artist {
  font-size: 11px;
  color: #94a3b8;
  margin-top: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 右键菜单 */
.context-menu {
  position: fixed;
  background-color: #1e293b;
  border: 1px solid #334155;
  border-radius: 8px;
  padding: 6px 0;
  min-width: 160px;
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

.context-menu-item.delete {
  color: #ef4444;
}

.context-menu-item.delete:hover {
  background-color: rgba(239, 68, 68, 0.15);
}

.context-menu-divider {
  height: 1px;
  background-color: rgba(255, 255, 255, 0.08);
  margin: 4px 0;
}

/* 添加歌曲弹窗样式 */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.modal-content {
  background: #1e293b;
  border-radius: 10px;
  width: 520px;
  max-width: 90vw;
  max-height: 70vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 10px 30px rgba(0,0,0,0.4);
}

.modal-title {
  font-size: 16px;
  font-weight: 600;
  margin: 0;
  padding: 16px 20px;
  border-bottom: 1px solid rgba(255,255,255,0.05);
}

.modal-search {
  padding: 12px 20px;
}

.modal-search-input {
  width: 100%;
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid rgba(255,255,255,0.1);
  background: #0f172a;
  color: #e2e8f0;
  font-size: 14px;
  outline: none;
}

.modal-search-input:focus {
  border-color: #3b82f6;
}

.modal-song-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 20px 12px;
}

.modal-song-item {
  display: grid;
  grid-template-columns: 32px 2fr 1fr;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 0.15s;
  font-size: 13px;
}

.modal-song-item:hover {
  background-color: rgba(255, 255, 255, 0.05);
}

.modal-song-item.selected {
  background-color: rgba(59, 130, 246, 0.15);
}

.check-box {
  width: 18px;
  height: 18px;
  border: 1.5px solid #64748b;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  color: white;
}

.modal-song-item.selected .check-box {
  background-color: #3b82f6;
  border-color: #3b82f6;
}

.modal-song-title {
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.modal-song-artist {
  color: #94a3b8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.modal-empty {
  text-align: center;
  color: #64748b;
  padding: 20px;
  font-size: 13px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 12px 20px;
  border-top: 1px solid rgba(255,255,255,0.05);
}

.modal-btn {
  padding: 7px 16px;
  border-radius: 6px;
  border: none;
  font-size: 13px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.modal-btn.cancel {
  background: rgba(255,255,255,0.08);
  color: #cbd5e1;
}

.modal-btn.cancel:hover {
  background: rgba(255,255,255,0.12);
}

.modal-btn.confirm {
  background: #3b82f6;
  color: white;
}

.modal-btn.confirm:hover {
  background: #2563eb;
}

.modal-btn.confirm:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
