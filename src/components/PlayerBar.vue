<template>
  <div class="player-bar">
    <div class="song-info">
      <div class="cover-placeholder">🎵</div>
      <div class="meta">
        <div class="title">{{ playerStore.currentSong?.title || "未在播放" }}</div>
        <div class="artist">{{ playerStore.currentSong?.artist || "-" }}</div>
      </div>
    </div>

    <div class="controls">
      <div class="control-buttons">
        <button class="control-btn" @click="playerStore.previous" title="上一首">⏮</button>
        <button class="play-btn" @click="playerStore.togglePlay" title="播放/暂停">
          {{ playerStore.isPlaying ? "⏸" : "▶" }}
        </button>
        <button class="control-btn" @click="playerStore.next" title="下一首">⏭</button>
      </div>
      <div class="progress-bar">
        <span class="time">{{ formatDuration(playerStore.progress) }}</span>
        <div class="progress-track" @click="seek">
          <div class="progress-fill" :style="{ width: progressPercent }"></div>
        </div>
        <span class="time">{{ formatDuration(playerStore.duration) }}</span>
      </div>
    </div>

    <div class="extra-controls">
      <button class="lyrics-btn" @click="goToLyrics" title="歌词">
        <span>📜</span>
        <span class="lyrics-label">歌词</span>
      </button>
      <button class="mode-btn" @click="cycleMode" title="播放模式">
        <span class="mode-icon">{{ modeIcon }}</span>
        <span class="mode-label">{{ modeLabel }}</span>
      </button>
      <div class="volume-control">
        <span>🔊</span>
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          :value="playerStore.volume"
          @input="(e) => playerStore.setVolume(Number((e.target as HTMLInputElement).value))"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import { usePlayerStore } from "../stores/player";
import { formatDuration } from "../utils/format";

const playerStore = usePlayerStore();
const router = useRouter();

function goToLyrics() {
  router.push({ name: "lyrics" });
}

const progressPercent = computed(() => {
  if (playerStore.duration === 0) return "0%";
  return `${(playerStore.progress / playerStore.duration) * 100}%`;
});

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

function seek(e: MouseEvent) {
  const track = e.currentTarget as HTMLDivElement;
  const rect = track.getBoundingClientRect();
  const ratio = (e.clientX - rect.left) / rect.width;
  playerStore.setProgress(ratio * playerStore.duration);
}

</script>

<style scoped>
.player-bar {
  height: 80px;
  background-color: #0f172a;
  border-top: 1px solid #1e293b;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
}

.song-info {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 240px;
}

.cover-placeholder {
  width: 48px;
  height: 48px;
  background-color: #1e293b;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
}

.meta {
  display: flex;
  flex-direction: column;
  gap: 4px;
  overflow: hidden;
}

.title {
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.artist {
  font-size: 12px;
  color: #94a3b8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.controls {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  flex: 1;
  max-width: 500px;
}

.control-buttons {
  display: flex;
  align-items: center;
  gap: 16px;
}

.control-btn {
  background: none;
  border: none;
  color: #94a3b8;
  font-size: 18px;
  cursor: pointer;
  padding: 4px;
  transition: color 0.2s;
}

.control-btn:hover {
  color: #e2e8f0;
}

.play-btn {
  width: 36px;
  height: 36px;
  background-color: #e2e8f0;
  border: none;
  border-radius: 50%;
  color: #0f172a;
  font-size: 16px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.2s;
}

.play-btn:hover {
  transform: scale(1.05);
}

.progress-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}

.time {
  font-size: 11px;
  color: #94a3b8;
  width: 36px;
  text-align: center;
}

.progress-track {
  flex: 1;
  height: 4px;
  background-color: #334155;
  border-radius: 2px;
  cursor: pointer;
  position: relative;
}

.progress-fill {
  height: 100%;
  background-color: #3b82f6;
  border-radius: 2px;
  transition: width 0.1s linear;
}

.extra-controls {
  display: flex;
  align-items: center;
  gap: 16px;
  width: 200px;
  justify-content: flex-end;
}

.lyrics-btn {
  background: none;
  border: none;
  color: #94a3b8;
  font-size: 14px;
  cursor: pointer;
  padding: 4px 8px;
  display: flex;
  align-items: center;
  gap: 4px;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.lyrics-btn:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.lyrics-label {
  font-size: 12px;
}

.mode-btn {
  background: none;
  border: none;
  color: #94a3b8;
  font-size: 12px;
  cursor: pointer;
  padding: 2px 6px;
  display: flex;
  align-items: center;
  gap: 2px;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.mode-btn:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.mode-icon {
  font-size: 14px;
}

.mode-label {
  font-size: 11px;
}

.volume-control {
  display: flex;
  align-items: center;
  gap: 8px;
}

.volume-control input[type="range"] {
  width: 80px;
}
</style>
