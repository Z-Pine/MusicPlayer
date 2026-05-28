<template>
  <div class="playlist-view">
    <!-- 歌单横幅 -->
    <header class="playlist-header-compact">
      <div class="header-left">
        <div class="playlist-icon-small">{{ isRecent ? '🕐' : '📁' }}</div>
        <div>
          <h1 class="playlist-title-compact">{{ playlistName }}</h1>
          <p class="playlist-subtitle">{{ songs.length }} 首歌曲</p>
        </div>
      </div>
      <div class="header-right" v-if="!isRecent && playlistId !== null">
        <button class="action-btn add" @click="openAddSongModal">添加歌曲</button>
        <button class="action-btn rename" @click="renamePlaylist">重命名</button>
        <button class="action-btn delete" @click="deletePlaylist">删除歌单</button>
      </div>
    </header>

    <!-- 播放控制区 -->
    <div class="player-control-wrapper" v-if="playerStore.currentSong">
      <div class="player-control-section">
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

    <div class="song-list-container">
      <!-- 歌曲列表工具栏 -->
      <div class="song-list-toolbar" v-if="songs.length > 0 && !isRecent">
        <button class="toolbar-btn select-all" @click="toggleSelectAll" :title="isAllSelected ? '取消全选' : '全选'">
          <span class="select-icon">{{ isAllSelected ? '☑' : '☐' }}</span>
          <span class="select-text">{{ isAllSelected ? '取消全选' : '全选' }}</span>
        </button>
        <span class="selected-count" v-if="selectedSongIds.size > 0">
          已选中 {{ selectedSongIds.size }} 首歌曲
        </span>
      </div>
      
      <div class="song-list" v-if="songs.length > 0">
        <div
          v-for="song in songs"
          :key="song.id"
          :id="'pl-song-' + song.id"
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
          <div class="song-actions">
            <button
              v-if="!isRecent"
              class="remove-btn"
              @click.stop="removeSongs(song.id)"
              :title="selectedSongIds.size > 0 ? `批量删除 ${selectedSongIds.size} 首` : '从歌单移除'"
            >
              ✕
            </button>
          </div>
        </div>
      </div>

      <div class="empty-tip" v-else>
        {{ isRecent ? '暂无播放记录，去音乐库听几首歌吧' : '歌单暂无歌曲，去音乐库添加吧' }}
      </div>
    </div>

    <!-- 上下文菜单 -->
    <div
      v-if="contextMenu.visible"
      class="context-menu"
      :style="{ top: contextMenu.y + 'px', left: contextMenu.x + 'px' }"
    >
      <div class="context-menu-item" @click="playSongFromContext">播放</div>
      <div class="context-menu-divider" v-if="!isRecent"></div>
      <div class="context-menu-item" @click="removeSongsFromContext" v-if="!isRecent">从歌单移除</div>
    </div>

    <!-- Toast 提示 -->
    <div v-if="toast.visible" class="toast" :class="{ show: toast.visible }">
      {{ toast.message }}
    </div>

    <!-- 添加歌曲弹窗 -->
    <div v-if="addModalOpen" class="modal-overlay" @click="closeAddSongModal" @keydown.esc="closeAddSongModal" @keydown.enter="confirmAddSongs">
      <div class="modal-content" @click.stop>
        <h3 class="modal-title">添加歌曲到「{{ playlistName }}」</h3>
        <div class="modal-search">
          <input
            v-model="addModalSearch"
            type="text"
            class="modal-search-input"
            placeholder="搜索歌曲..."
            @input="onAddModalSearch"
            @keydown.esc="closeAddSongModal"
            @keydown.enter.prevent="confirmAddSongs"
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { usePlayerStore } from "../stores/player";
import type { Song } from "../stores/player";
import { formatDuration } from "../utils/format";
import { useDialog } from "../composables/useDialog";

const route = useRoute();
const router = useRouter();
const playerStore = usePlayerStore();
const dialog = useDialog();

const songs = ref<Song[]>([]);
const playlistName = ref("");
const isLoading = ref(false);

// 添加歌曲弹窗相关
const addModalOpen = ref(false);
const addModalSearch = ref("");
const addModalSongs = ref<Song[]>([]);
const addModalSelected = ref<number[]>([]);
const addModalSearchTimer = ref<ReturnType<typeof setTimeout> | null>(null);
const addModalExistingSongIds = ref<Set<number>>(new Set());
const addModalLoading = ref(false);
const addModalError = ref("");
const addModalSearchInput = ref<HTMLInputElement | null>(null);

// 多选歌曲
const selectedSongIds = ref<Set<number>>(new Set());
const lastClickedId = ref<number | null>(null);

// 上下文菜单
const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  song: null as Song | null,
});

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

const isRecent = computed(() => route.params.id === "recent");
const playlistId = computed(() => {
  if (isRecent.value) return null;
  const id = Number(route.params.id);
  return isNaN(id) ? null : id;
});

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

async function fetchSongs() {
  isLoading.value = true;
  try {
    if (isRecent.value) {
      // 最近播放
      const result = await invoke<{
        song_id: number;
        played_at: number;
        song: Song | null;
      }[]>("get_recent_plays", { limit: 100 });
      songs.value = result
        .map((r) => r.song)
        .filter((s): s is Song => s !== null);
      playlistName.value = "最近播放";
    } else if (playlistId.value !== null) {
      // 普通歌单
      const result = await invoke<{
        playlist_id: number;
        song_id: number;
        position: number;
        song: Song | null;
      }[]>("get_playlist_songs", { playlistId: playlistId.value });
      songs.value = result
        .map((r) => r.song)
        .filter((s): s is Song => s !== null);
      // 获取歌单名称
      const playlists = await invoke<{ id: number; name: string }[]>("get_playlists");
      const pl = playlists.find((p) => p.id === playlistId.value);
      playlistName.value = pl?.name || "歌单详情";
    }
  } catch (e) {
    console.error("Failed to fetch songs:", e);
  } finally {
    isLoading.value = false;
  }
}

function playSong(song: Song) {
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

// 全选功能
const isAllSelected = computed(() => {
  if (songs.value.length === 0) return false;
  return selectedSongIds.value.size === songs.value.length;
});

function toggleSelectAll() {
  if (isAllSelected.value) {
    // 取消全选
    selectedSongIds.value.clear();
  } else {
    // 全选
    selectedSongIds.value.clear();
    songs.value.forEach(song => {
      selectedSongIds.value.add(song.id);
    });
  }
}

async function removeSongs(songId?: number) {
  if (isRecent.value) return; // 最近播放不可删除
  if (playlistId.value === null) return;
  
  // 确定要删除的歌曲列表
  let idsToRemove: number[];
  
  if (songId !== undefined) {
    // 如果提供了 songId（从删除按钮调用）
    // 检查该歌曲是否在选中列表中
    if (selectedSongIds.value.has(songId) && selectedSongIds.value.size > 1) {
      // 如果该歌曲在选中列表中，且选中了多首歌曲，则删除所有选中的（批量删除）
      idsToRemove = Array.from(selectedSongIds.value);
    } else {
      // 否则只删除该歌曲（单个删除）
      idsToRemove = [songId];
    }
  } else if (selectedSongIds.value.size > 0) {
    // 如果没有提供 songId（从右键菜单调用），删除所有选中的歌曲
    idsToRemove = Array.from(selectedSongIds.value);
  } else {
    // 既没有提供 songId，也没有选中的歌曲，直接返回
    return;
  }

  const confirmed = await dialog.confirm({
    title: '确认删除',
    message: `确定要从歌单中移除 ${idsToRemove.length} 首歌曲吗？`,
    type: 'warning',
    confirmText: '删除',
    cancelText: '取消',
  });
  
  if (!confirmed) return;

  try {
    for (const songId of idsToRemove) {
      await invoke("remove_song_from_playlist", {
        playlistId: playlistId.value,
        songId: songId,
      });
    }
    selectedSongIds.value.clear();
    await fetchSongs();
    showToast(`已从歌单中移除 ${idsToRemove.length} 首歌曲`);
  } catch (e) {
    console.error("Failed to remove songs:", e);
    await dialog.error("删除失败: " + String(e));
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
}

function playSongFromContext() {
  if (contextMenu.value.song) {
    playSong(contextMenu.value.song);
  }
  closeContextMenu();
}

async function removeSongsFromContext() {
  closeContextMenu();
  await removeSongs();
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

async function renamePlaylist() {
  if (playlistId.value === null) return;
  const newName = await dialog.prompt('', '重命名歌单', playlistName.value, '请输入新歌单名称');
  if (newName && newName.trim()) {
    try {
      await invoke("rename_playlist", {
        id: playlistId.value,
        name: newName.trim(),
      });
      playlistName.value = newName.trim();
      // 通知侧边栏刷新歌单列表
      window.dispatchEvent(new CustomEvent("playlists-updated"));
    } catch (e) {
      console.error("Failed to rename playlist:", e);
      await dialog.error("重命名失败: " + String(e));
    }
  }
}

async function deletePlaylist() {
  if (playlistId.value === null) return;
  
  const confirmed = await dialog.confirm({
    title: '确认删除',
    message: `确定要删除歌单「${playlistName.value}」吗？`,
    type: 'warning',
    confirmText: '删除',
    cancelText: '取消',
  });
  
  if (!confirmed) return;
  
  try {
    await invoke("delete_playlist", { id: playlistId.value });
    // 通知侧边栏刷新歌单列表
    window.dispatchEvent(new CustomEvent("playlists-updated"));
    router.push("/");
  } catch (e) {
    console.error("Failed to delete playlist:", e);
    await dialog.error("删除失败: " + String(e));
  }
}

// 添加歌曲弹窗逻辑
async function openAddSongModal() {
  if (playlistId.value === null) return;
  addModalOpen.value = true;
  addModalSearch.value = "";
  addModalSelected.value = [];
  addModalSongs.value = [];
  // 记录歌单中已有的歌曲ID，用于去重
  addModalExistingSongIds.value = new Set(songs.value.map((s) => s.id));
  // 自动加载全部歌曲
  await loadAddModalSongs();
  // 自动聚焦到搜索框
  setTimeout(() => {
    addModalSearchInput.value?.focus();
  }, 100);
}

function closeAddSongModal() {
  addModalOpen.value = false;
  addModalSearch.value = "";
  addModalSelected.value = [];
  addModalSongs.value = [];
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
  if (playlistId.value === null || addModalSelected.value.length === 0) return;
  try {
    for (const songId of addModalSelected.value) {
      await invoke("add_song_to_playlist", {
        playlistId: playlistId.value,
        songId: songId,
      });
    }
    closeAddSongModal();
    await fetchSongs();
  } catch (e) {
    console.error("Failed to add songs:", e);
    alert("添加失败: " + String(e));
  }
}

// 监听路由参数变化
watch(
  () => route.params.id,
  () => {
    // 清空当前状态
    songs.value = [];
    selectedSongIds.value.clear();
    lastClickedId.value = null;
    // 重新加载数据
    fetchSongs();
  }
);

function onPlaylistSongsUpdated() {
  // 当歌单歌曲更新时，重新加载歌曲列表
  fetchSongs();
}

onMounted(() => {
  fetchSongs();
  // 监听歌单更新事件
  window.addEventListener("playlists-updated", onPlaylistSongsUpdated);
});

onUnmounted(() => {
  // 移除事件监听
  window.removeEventListener("playlists-updated", onPlaylistSongsUpdated);
});
</script>

<style scoped>
.playlist-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
}

/* 固定头部区域 */
.playlist-view > :not(.song-list-container):not(.modal-overlay) {
  flex-shrink: 0;
  margin: 0 24px;
}

.playlist-view > .playlist-header-compact {
  margin-top: 24px;
}

.playlist-view > .player-control-wrapper {
  margin-top: 20px;
}

/* 歌单横幅 */
.playlist-header-compact {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.08) 0%, rgba(30, 41, 59, 0.4) 100%);
  border-radius: 10px;
  gap: 20px;
  flex-wrap: wrap;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 14px;
  flex-shrink: 0;
}

.playlist-icon-small {
  width: 50px;
  height: 50px;
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  flex-shrink: 0;
}

.playlist-title-compact {
  font-size: 24px;
  font-weight: 700;
  margin: 0;
  color: #f8fafc;
  line-height: 1.2;
}

.playlist-subtitle {
  font-size: 12px;
  color: #94a3b8;
  margin: 2px 0 0 0;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.action-btn {
  padding: 6px 14px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.action-btn.add {
  background-color: #10b981;
  color: white;
}

.action-btn.add:hover {
  background-color: #059669;
}

.action-btn.rename {
  background-color: #3b82f6;
  color: white;
}

.action-btn.rename:hover {
  background-color: #2563eb;
}

.action-btn.delete {
  background-color: #ef4444;
  color: white;
}

.action-btn.delete:hover {
  background-color: #dc2626;
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
  border-radius: 10px;
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

/* 歌曲列表工具栏 */
.song-list-toolbar {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 16px;
  background: rgba(30, 41, 59, 0.5);
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.toolbar-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
  background: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
  border: 1px solid rgba(59, 130, 246, 0.3);
}

.toolbar-btn:hover {
  background: rgba(59, 130, 246, 0.2);
  border-color: rgba(59, 130, 246, 0.5);
}

.toolbar-btn .select-icon {
  font-size: 16px;
}

.toolbar-btn .select-text {
  font-weight: 500;
}

.selected-count {
  font-size: 13px;
  color: #94a3b8;
  margin-left: auto;
}

.song-list {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.song-item {
  display: grid;
  grid-template-columns: 36px 2fr 1fr 1fr 80px 40px;
  padding: 12px 16px;
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 0.2s;
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

.song-index {
  color: #64748b;
  font-size: 14px;
  text-align: center;
}

.playing-icon {
  color: #3b82f6;
  font-size: 10px;
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

.song-actions {
  text-align: center;
  opacity: 0;
  transition: opacity 0.2s;
}

.song-item:hover .song-actions {
  opacity: 1;
}

.remove-btn {
  background: none;
  border: none;
  color: #94a3b8;
  cursor: pointer;
  font-size: 14px;
  padding: 4px 8px;
  border-radius: 4px;
}

.remove-btn:hover {
  color: #ef4444;
  background-color: rgba(239, 68, 68, 0.1);
}

.empty-tip {
  padding: 60px;
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
  max-width: 180px;
  width: 180px;
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
  max-width: 120px;
}

.player-song-title {
  font-size: 14px;
  font-weight: 600;
  color: #e2e8f0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}

.player-song-artist {
  font-size: 12px;
  color: #94a3b8;
  margin-top: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
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
  z-index: 1000;
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

/* 上下文菜单 */
.context-menu {
  position: fixed;
  background-color: #1e293b;
  border: 1px solid #334155;
  border-radius: 8px;
  padding: 6px 0;
  min-width: 180px;
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

.context-menu-divider {
  height: 1px;
  background-color: rgba(255, 255, 255, 0.08);
  margin: 4px 0;
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
