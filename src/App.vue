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
import { invoke } from '@tauri-apps/api/core';

const dialogRef = ref();
const playerStore = usePlayerStore();

// 定期保存播放状态（每10秒）
let saveInterval: ReturnType<typeof setInterval> | null = null;

onMounted(async () => {
  setDialogRef(dialogRef.value);
  
  // 禁用默认右键菜单
  document.addEventListener('contextmenu', (e) => {
    // 只在非输入框元素上禁用
    const target = e.target as HTMLElement;
    if (target.tagName !== 'INPUT' && target.tagName !== 'TEXTAREA') {
      // 如果不是自定义右键菜单触发的，则阻止默认行为
      if (!target.closest('.song-item') && !target.closest('.context-menu')) {
        e.preventDefault();
      }
    }
  });
  
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

  // 启动 5 秒后后台检查文件完整性（不影响 UI 性能）
  setTimeout(async () => {
    try {
      await invoke("check_and_mark_invalid_files");
    } catch (e) {
      // 静默处理，不阻塞用户体验
      console.debug("Background file check skipped:", e);
    }
  }, 5000);
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
