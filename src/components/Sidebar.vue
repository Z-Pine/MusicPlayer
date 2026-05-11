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
      <div class="nav-item" @click="openRecentPlays" style="cursor: pointer;">
        <span class="nav-icon">🕐</span>
        <span>最近播放</span>
      </div>
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
          @click="openPlaylist(pl)"
        >
          {{ pl.name }}
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import type { PlaylistItem } from "../stores/player";

const router = useRouter();
const playlists = ref<PlaylistItem[]>([]);

async function fetchPlaylists() {
  try {
    const result = await invoke<PlaylistItem[]>("get_playlists");
    playlists.value = result;
  } catch (e) {
    console.error("Failed to fetch playlists:", e);
  }
}

async function createPlaylist() {
  const name = prompt("请输入歌单名称");
  if (name && name.trim()) {
    try {
      await invoke("create_playlist", { name: name.trim() });
      await fetchPlaylists();
    } catch (e) {
      console.error("Failed to create playlist:", e);
    }
  }
}

function openPlaylist(pl: PlaylistItem) {
  router.push(`/playlist/${pl.id}`);
}

function openRecentPlays() {
  router.push("/playlist/recent");
}

onMounted(() => {
  fetchPlaylists();
});
</script>

<style scoped>
.sidebar {
  width: 220px;
  background-color: #0f172a;
  border-right: 1px solid #1e293b;
  display: flex;
  flex-direction: column;
  padding: 20px 16px;
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 32px;
  padding: 0 8px;
}

.logo-icon {
  font-size: 24px;
}

.logo-text {
  font-size: 18px;
  font-weight: 700;
  color: #f8fafc;
}

.nav-menu {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 24px;
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
  font-size: 16px;
}

.playlists-section {
  flex: 1;
  overflow-y: auto;
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
  font-size: 18px;
  cursor: pointer;
  padding: 0 4px;
}

.add-btn:hover {
  color: #e2e8f0;
}

.playlist-list {
  display: flex;
  flex-direction: column;
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
</style>
