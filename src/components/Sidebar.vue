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
        >
          📁 {{ pl.name }}
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
</style>
