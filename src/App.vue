<template>
  <router-view />
  <ConfirmDialog ref="dialogRef" />
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import ConfirmDialog from './components/ConfirmDialog.vue';
import { setDialogRef } from './composables/useDialog';
import { usePlayerStore } from './stores/player';
import { listen } from '@tauri-apps/api/event';

const dialogRef = ref();
const playerStore = usePlayerStore();

// 定期保存播放状态（每10秒）
let saveInterval: ReturnType<typeof setInterval> | null = null;

onMounted(async () => {
  setDialogRef(dialogRef.value);
  
  // 加载上次的播放状态
  await playerStore.restorePlaybackState();
  
  // 定期保存播放状态
  saveInterval = setInterval(() => {
    if (playerStore.currentSong) {
      playerStore.savePlaybackState();
    }
  }, 10000); // 每10秒保存一次
  
  // 监听窗口关闭事件
  const unlisten = await listen('tauri://close-requested', async () => {
    // 保存播放状态
    await playerStore.savePlaybackState();
  });
  
  // 清理监听器
  onBeforeUnmount(() => {
    unlisten();
  });
});

onBeforeUnmount(() => {
  // 清理定时器
  if (saveInterval) {
    clearInterval(saveInterval);
    saveInterval = null;
  }
  
  // 保存播放状态
  playerStore.savePlaybackState();
  
  // 清理播放器资源
  playerStore.dispose();
});
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  width: 100%;
  height: 100%;
  overflow: hidden;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}
</style>
