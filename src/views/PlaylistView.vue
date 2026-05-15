<template>
  <div class="playlist-view">
    <header class="playlist-header">
      <div class="playlist-info">
        <div class="playlist-icon">{{ isRecent ? '🕐' : '📁' }}</div>
        <div class="meta">
          <h1 class="playlist-title">{{ playlistName }}</h1>
          <p class="playlist-desc">{{ songs.length }} 首歌曲</p>
        </div>
      </div>
      <div class="playlist-actions" v-if="!isRecent && playlistId !== null">
        <button class="action-btn add" @click="openAddSongModal">添加歌曲</button>
        <button class="action-btn rename" @click="renamePlaylist">重命名</button>
        <button class="action-btn delete" @click="deletePlaylist">删除歌单</button>
      </div>
    </header>

    <div class="song-list" v-if="songs.length > 0">
      <div
        v-for="(song, index) in songs"
        :key="song.id"
        :id="'pl-song-' + song.id"
        class="song-item"
        :class="{ playing: playerStore.currentSong?.id === song.id }"
        @dblclick="playSong(song)"
      >
        <div class="song-index">
          <span v-if="playerStore.currentSong?.id === song.id" class="playing-icon">▶</span>
          <span v-else>{{ index + 1 }}</span>
        </div>
        <div class="song-title">{{ song.title }}</div>
        <div class="song-artist">{{ song.artist }}</div>
        <div class="song-album">{{ song.album }}</div>
        <div class="song-duration">{{ formatDuration(song.duration) }}</div>
        <div class="song-actions">
          <button
            class="remove-btn"
            @click.stop="removeSong(song.id)"
            title="从歌单移除"
          >
            ✕
          </button>
        </div>
      </div>
    </div>

    <div class="empty-tip" v-else>
      {{ isRecent ? '暂无播放记录，去音乐库听几首歌吧' : '歌单暂无歌曲，去音乐库添加吧' }}
    </div>

    <!-- 添加歌曲弹窗 -->
    <div v-if="addModalOpen" class="modal-overlay" @click="closeAddSongModal">
      <div class="modal-content" @click.stop>
        <h3 class="modal-title">添加歌曲到歌单</h3>
        <div class="modal-search">
          <input
            v-model="addModalSearch"
            type="text"
            class="modal-search-input"
            placeholder="搜索歌曲..."
            @input="onAddModalSearch"
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
import { ref, computed, onMounted } from "vue";
import { useRoute, useRouter, onBeforeRouteUpdate } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { usePlayerStore } from "../stores/player";
import type { Song } from "../stores/player";
import { formatDuration } from "../utils/format";

const route = useRoute();
const router = useRouter();
const playerStore = usePlayerStore();

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

const isRecent = computed(() => route.params.id === "recent");
const playlistId = computed(() => {
  if (isRecent.value) return null;
  const id = Number(route.params.id);
  return isNaN(id) ? null : id;
});

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

async function removeSong(songId: number) {
  if (isRecent.value) return; // 最近播放不可删除
  if (playlistId.value === null) return;
  try {
      await invoke("remove_song_from_playlist", {
        playlistId: playlistId.value,
        songId: songId,
      });
    await fetchSongs();
  } catch (e) {
    console.error("Failed to remove song:", e);
  }
}

async function renamePlaylist() {
  if (playlistId.value === null) return;
  const newName = prompt("请输入新歌单名称", playlistName.value);
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
    }
  }
}

async function deletePlaylist() {
  if (playlistId.value === null) return;
  if (!confirm(`确定要删除歌单「${playlistName.value}」吗？`)) return;
  try {
    await invoke("delete_playlist", { id: playlistId.value });
    router.push("/");
  } catch (e) {
    console.error("Failed to delete playlist:", e);
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

// 路由参数变化时重新加载
onBeforeRouteUpdate(() => {
  fetchSongs();
});

onMounted(() => {
  fetchSongs();
});
</script>

<style scoped>
.playlist-view {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
}

.playlist-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.playlist-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.playlist-icon {
  width: 64px;
  height: 64px;
  background-color: #1e293b;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 28px;
}

.meta {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.playlist-title {
  font-size: 24px;
  font-weight: 600;
  margin: 0;
}

.playlist-desc {
  font-size: 14px;
  color: #94a3b8;
  margin: 0;
}

.playlist-actions {
  display: flex;
  gap: 8px;
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

.song-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.song-item {
  display: grid;
  grid-template-columns: 40px 2fr 1fr 1fr 80px 40px;
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

.song-item.playing {
  background-color: rgba(59, 130, 246, 0.15);
  border-left: 3px solid #3b82f6;
}

.song-item.playing .song-title {
  color: #3b82f6;
  font-weight: 600;
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
}

.song-artist,
.song-album {
  color: #94a3b8;
  font-size: 14px;
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
</style>
