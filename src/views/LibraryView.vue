<template>
  <div class="library-view">
    <header class="library-header">
      <h1>音乐库</h1>
      <button class="scan-btn" @click="scanLibrary" :disabled="scanning">
        {{ scanning ? "扫描中..." : "扫描文件夹" }}
      </button>
    </header>
    <div v-if="errorMsg" class="error-msg">{{ errorMsg }}</div>
    <div class="song-list">
      <div
        v-for="song in songs"
        :key="song.id"
        :id="'song-' + song.id"
        class="song-item"
        :class="{ playing: playerStore.currentSong?.id === song.id }"
        @dblclick="playSong(song)"
      >
        <div class="song-title">
          <span v-if="playerStore.currentSong?.id === song.id" class="playing-icon">▶</span>
          {{ song.title }}
        </div>
        <div class="song-artist">{{ song.artist }}</div>
        <div class="song-album">{{ song.album }}</div>
        <div class="song-duration">{{ formatDuration(song.duration) }}</div>
      </div>
      <div v-if="songs.length === 0 && !scanning" class="empty-tip">
        暂无歌曲，点击「扫描文件夹」开始添加音乐
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { usePlayerStore } from "../stores/player";
import type { Song } from "../stores/player";
import { formatDuration } from "../utils/format";

const playerStore = usePlayerStore();
const songs = ref<Song[]>([]);
const scanning = ref(false);
const errorMsg = ref("");

async function fetchSongs() {
  try {
    const result = await invoke<Song[]>("get_all_songs");
    songs.value = result;
  } catch (e) {
    console.error("Failed to fetch songs:", e);
    errorMsg.value = "获取歌曲列表失败: " + String(e);
  }
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
  playerStore.playSong(song, songs.value);
}

watch(
  () => playerStore.currentSong,
  (newSong) => {
    if (newSong) {
      nextTick(() => {
        const el = document.getElementById("song-" + newSong.id);
        if (el) {
          const rect = el.getBoundingClientRect();
          const container = document.querySelector('.library-view');
          if (container) {
            const containerRect = container.getBoundingClientRect();
            const isVisible = (
              rect.top >= containerRect.top &&
              rect.bottom <= containerRect.bottom
            );
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
});
</script>

<style scoped>
.library-view {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
}

.library-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.library-header h1 {
  font-size: 24px;
  font-weight: 600;
}

.scan-btn {
  padding: 8px 16px;
  background-color: #3b82f6;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
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
  grid-template-columns: 2fr 1fr 1fr 80px;
  padding: 12px 16px;
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 0.2s;
  user-select: none;
  -webkit-user-select: none;
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

.playing-icon {
  color: #3b82f6;
  font-size: 10px;
  margin-right: 8px;
}

.song-title {
  font-weight: 500;
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

.empty-tip {
  padding: 40px;
  text-align: center;
  color: #64748b;
  font-size: 14px;
}
</style>
