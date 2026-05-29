<template>
  <Teleport to="body">
    <Transition name="dialog-fade">
      <div v-if="visible" class="modal-overlay" @click="close">
        <Transition name="dialog-scale">
          <div v-if="visible" class="modal-content" @click.stop>
            <div class="modal-header">
              <div class="header-icon">🎵</div>
              <h3 class="header-title">歌曲属性</h3>
              <button class="close-btn" @click="close">✕</button>
            </div>

            <div class="modal-body">
              <!-- 基本信息 -->
              <section class="info-section">
                <div class="info-row">
                  <span class="info-label">标题</span>
                  <span class="info-value">{{ song?.title || '-' }}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">艺术家</span>
                  <span class="info-value">{{ song?.artist || '-' }}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">专辑</span>
                  <span class="info-value">{{ song?.album || '-' }}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">专辑艺术家</span>
                  <span class="info-value">{{ song?.albumArtist || '-' }}</span>
                </div>
              </section>

              <div class="section-divider"></div>

              <!-- 技术信息 -->
              <section class="info-section">
                <div class="info-row">
                  <span class="info-label">时长</span>
                  <span class="info-value">{{ formatDuration(song?.duration || 0) }}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">比特率</span>
                  <span class="info-value">{{ song?.bitrate ? song.bitrate + ' kbps' : '-' }}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">采样率</span>
                  <span class="info-value">{{ song?.sampleRate ? song.sampleRate + ' Hz' : '-' }}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">年代</span>
                  <span class="info-value">{{ song?.year ?? '-' }}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">流派</span>
                  <span class="info-value">{{ song?.genre || '-' }}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">文件大小</span>
                  <span class="info-value">{{ fileSizeDisplay }}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">修改时间</span>
                  <span class="info-value">{{ modifiedTimeDisplay }}</span>
                </div>
              </section>

              <div class="section-divider"></div>

              <!-- 文件路径 -->
              <section class="info-section">
                <div class="path-header">
                  <span class="info-label">文件路径</span>
                  <button class="locate-btn" @click="openFileLocation" title="在资源管理器中定位该文件">
                    📂 打开位置
                  </button>
                </div>
                <div class="path-display">{{ song?.path || '-' }}</div>
              </section>

              <div class="section-divider"></div>

              <!-- 状态 -->
              <section class="info-section">
                <div class="info-row">
                  <span class="info-label">状态</span>
                  <span class="info-value" :class="statusClass">{{ statusText }}</span>
                </div>
              </section>
            </div>

            <div class="modal-footer">
              <button class="close-btn-primary" @click="close">关闭</button>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Song } from "../stores/player";
import { formatDuration } from "../utils/format";

const props = defineProps<{
  visible: boolean;
  song: Song | null;
}>();

const emit = defineEmits<{
  close: [];
}>();

const fileInfo = ref<{ size: number; modified: number | null; created: number | null } | null>(null);

function close() {
  emit("close");
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Escape" && props.visible) {
    e.preventDefault();
    close();
  }
}

// 打开资源管理器定位文件（fire-and-forget，不等待 IPC 返回）
function openFileLocation() {
  if (!props.song?.path) return;
  invoke("open_file_location", { path: props.song.path }).catch(console.error);
}

// 监听对话框显示状态，添加/移除键盘事件
watch(
  () => props.visible,
  (newVal) => {
    if (newVal) {
      document.addEventListener("keydown", handleKeydown);
      // 打开时自动加载文件信息
      if (props.song) loadFileInfo();
    } else {
      document.removeEventListener("keydown", handleKeydown);
    }
  }
);

// 组件卸载时清理
onUnmounted(() => {
  document.removeEventListener("keydown", handleKeydown);
});

async function loadFileInfo() {
  fileInfo.value = null;
  if (!props.song?.path) return;
  try {
    const info = await invoke<{ size: number; modified: number | null; created: number | null }>(
      "get_file_info",
      { path: props.song.path }
    );
    fileInfo.value = info;
  } catch {
    // 静默处理
  }
}

const fileSizeDisplay = computed(() => {
  if (!fileInfo.value) return "读取中...";
  const bytes = fileInfo.value.size;
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  return (bytes / (1024 * 1024)).toFixed(1) + " MB";
});

function formatTimestamp(secs: number | null): string {
  if (secs === null || secs === undefined) return "-";
  const date = new Date(secs * 1000);
  const y = date.getFullYear();
  const M = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  const h = String(date.getHours()).padStart(2, "0");
  const m = String(date.getMinutes()).padStart(2, "0");
  return `${y}-${M}-${d} ${h}:${m}`;
}

const modifiedTimeDisplay = computed(() => {
  if (!fileInfo.value) return "读取中...";
  return formatTimestamp(fileInfo.value.modified);
});

const statusText = computed(() => {
  if (!props.song) return "-";
  if (props.song.deletedFromLibrary) return "已从音乐库删除";
  if (!props.song.isAvailable) return "文件缺失";
  return "正常";
});

const statusClass = computed(() => {
  if (!props.song) return "";
  if (props.song.deletedFromLibrary || !props.song.isAvailable) return "status-warning";
  return "status-ok";
});
</script>

<style scoped>
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
  z-index: 9999;
}

.modal-content {
  background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
  border-radius: 14px;
  width: 480px;
  max-width: 90vw;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 15px 40px rgba(0, 0, 0, 0.5);
  border: 1px solid rgba(59, 130, 246, 0.15);
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 18px 22px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.header-icon {
  font-size: 22px;
}

.header-title {
  flex: 1;
  font-size: 17px;
  font-weight: 600;
  margin: 0;
  color: #f1f5f9;
}

.close-btn {
  background: none;
  border: none;
  color: #64748b;
  font-size: 16px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.15s;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #e2e8f0;
}

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 6px 0;
}

.info-section {
  padding: 12px 22px;
}

.info-row {
  display: flex;
  align-items: baseline;
  padding: 6px 0;
  gap: 16px;
}

.info-label {
  flex-shrink: 0;
  width: 80px;
  font-size: 13px;
  color: #94a3b8;
  font-weight: 500;
}

.info-value {
  flex: 1;
  font-size: 13px;
  color: #e2e8f0;
  word-break: break-all;
  line-height: 1.4;
}

.section-divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.05);
  margin: 4px 22px;
}

.path-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.locate-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border: none;
  border-radius: 5px;
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  background: rgba(59, 130, 246, 0.15);
  color: #60a5fa;
  transition: all 0.15s;
}

.locate-btn:hover {
  background: rgba(59, 130, 246, 0.25);
  color: #93c5fd;
}

.path-display {
  font-size: 12px;
  color: #94a3b8;
  word-break: break-all;
  padding: 8px 10px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 6px;
  font-family: monospace;
  line-height: 1.5;
  user-select: text;
}

.status-ok {
  color: #34d399;
}

.status-warning {
  color: #fbbf24;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  padding: 14px 22px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.close-btn-primary {
  padding: 8px 28px;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  color: white;
  transition: all 0.2s;
}

.close-btn-primary:hover {
  background: linear-gradient(135deg, #2563eb 0%, #1d4ed8 100%);
  transform: translateY(-1px);
}

/* 动画 */
.dialog-fade-enter-active,
.dialog-fade-leave-active {
  transition: opacity 0.25s ease;
}
.dialog-fade-enter-from,
.dialog-fade-leave-to {
  opacity: 0;
}
.dialog-scale-enter-active {
  transition: all 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
}
.dialog-scale-leave-active {
  transition: all 0.2s ease-out;
}
.dialog-scale-enter-from {
  opacity: 0;
  transform: scale(0.9) translateY(-15px);
}
.dialog-scale-leave-to {
  opacity: 0;
  transform: scale(0.95);
}
</style>
