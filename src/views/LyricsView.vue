<template>
  <div class="lyrics-view">
    <div class="lyrics-header">
      <div class="song-info">
        <div class="cover-placeholder">🎵</div>
        <div class="meta">
          <div class="title">{{ currentSong?.title || "未在播放" }}</div>
          <div class="artist">{{ currentSong?.artist || "-" }}</div>
        </div>
      </div>
    </div>
    <div ref="lyricsContainer" class="lyrics-container">
      <div v-if="lyrics.length === 0" class="no-lyrics">
        暂无歌词
      </div>
      <div
        v-for="(line, index) in lyrics"
        :key="index"
        class="lyric-line"
        :class="{ active: currentLineIndex === index }"
      >
        {{ line.text }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { usePlayerStore } from "../stores/player";

interface LyricLine {
  time: number;
  text: string;
}

const playerStore = usePlayerStore();
const lyrics = ref<LyricLine[]>([]);
const lyricsContainer = ref<HTMLDivElement | null>(null);
const progress = ref(0);
let progressTimer: ReturnType<typeof setInterval> | null = null;

const currentSong = computed(() => playerStore.currentSong);

// 当前应高亮的歌词行索引
const currentLineIndex = computed(() => {
  if (lyrics.value.length === 0) return -1;
  const currentTime = progress.value;
  for (let i = lyrics.value.length - 1; i >= 0; i--) {
    if (lyrics.value[i].time <= currentTime) {
      return i;
    }
  }
  return 0;
});

// 加载歌词
async function loadLyrics() {
  if (!currentSong.value) {
    lyrics.value = [];
    return;
  }
  try {
    const result = await invoke<LyricLine[]>("get_lyrics", {
      songId: currentSong.value.id,
    });
    lyrics.value = result;
  } catch (e) {
    console.error("Failed to load lyrics:", e);
    lyrics.value = [];
  }
}

// 自动滚动到当前歌词
function scrollToActiveLine() {
  if (!lyricsContainer.value) return;
  const activeEl = lyricsContainer.value.querySelector(".lyric-line.active") as HTMLElement;
  if (activeEl) {
    const container = lyricsContainer.value;
    const containerHeight = container.clientHeight;
    const lineTop = activeEl.offsetTop;
    const lineHeight = activeEl.clientHeight;
    container.scrollTo({
      top: lineTop - containerHeight / 2 + lineHeight / 2,
      behavior: "smooth",
    });
  }
}

// 监听歌曲变化，加载歌词
watch(currentSong, () => {
  loadLyrics();
});

// 监听当前行变化，滚动
watch(currentLineIndex, () => {
  nextTick(() => {
    scrollToActiveLine();
  });
});

// 同步播放进度
function startProgressSync() {
  stopProgressSync();
  progressTimer = setInterval(() => {
    progress.value = playerStore.progress;
  }, 500);
}

function stopProgressSync() {
  if (progressTimer) {
    clearInterval(progressTimer);
    progressTimer = null;
  }
}

onMounted(() => {
  loadLyrics();
  startProgressSync();
});

onUnmounted(() => {
  stopProgressSync();
});
</script>

<style scoped>
.lyrics-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.lyrics-header {
  padding: 24px 24px 16px;
  display: flex;
  align-items: center;
  gap: 16px;
  border-bottom: 1px solid #1e293b;
}

.song-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.cover-placeholder {
  width: 56px;
  height: 56px;
  background-color: #1e293b;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
}

.meta {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.meta .title {
  font-size: 18px;
  font-weight: 600;
}

.meta .artist {
  font-size: 14px;
  color: #94a3b8;
}

.lyrics-container {
  flex: 1;
  overflow-y: auto;
  padding: 32px 24px;
  text-align: center;
  scroll-behavior: smooth;
}

.no-lyrics {
  padding-top: 40px;
  color: #64748b;
  font-size: 14px;
}

.lyric-line {
  padding: 10px 0;
  font-size: 15px;
  color: #94a3b8;
  line-height: 1.6;
  transition: color 0.3s, transform 0.3s, font-weight 0.3s;
}

.lyric-line.active {
  color: #e2e8f0;
  font-weight: 600;
  font-size: 17px;
  transform: scale(1.05);
}
</style>
