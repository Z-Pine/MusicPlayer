<template>
  <div class="settings-view">
    <h1>设置</h1>
    <div class="settings-section">
      <h2>音乐库</h2>
      <div class="setting-item">
        <label>音乐文件夹路径</label>
        <div class="path-input">
          <button @click="selectDir">选择文件夹</button>
        </div>
      </div>
      <div class="source-list" v-if="sources.length > 0">
        <div v-for="src in sources" :key="src.id" class="source-item">
          <span class="source-path">{{ src.path }}</span>
          <button class="remove-btn" @click="removeSource(src.id)">删除</button>
        </div>
      </div>
      <div class="empty-tip" v-else>暂无音乐库源，请点击「选择文件夹」添加</div>
    </div>
    <div class="settings-section">
      <h2>播放</h2>
      <div class="setting-item">
        <label>默认音量</label>
        <input type="range" min="0" max="1" step="0.01" v-model="defaultVolume" />
        <span>{{ Math.round(defaultVolume * 100) }}%</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useDialog } from "../composables/useDialog";

interface LibrarySource {
  id: number;
  path: string;
  enabled: boolean;
  createdAt: number;
}

const defaultVolume = ref(0.8);
const sources = ref<LibrarySource[]>([]);
const dialog = useDialog();

async function fetchSources() {
  try {
    const result = await invoke<LibrarySource[]>("get_library_sources");
    sources.value = result;
  } catch (e) {
    console.error("Failed to fetch sources:", e);
  }
}

async function selectDir() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (selected && typeof selected === "string") {
      await invoke("add_library_source", { path: selected });
      await fetchSources();
    }
  } catch (e) {
    console.error("Failed to select directory:", e);
  }
}

async function removeSource(id: number) {
  // 获取要删除的源路径用于确认对话框
  const source = sources.value.find(s => s.id === id);
  const sourcePath = source ? source.path : '该音乐库源';
  
  const confirmed = await dialog.warning(
    `确定要删除音乐库源「${sourcePath}」吗？\n\n注意：这不会删除实际的音乐文件，只是从音乐库列表中移除。`,
    '确认删除音乐库源'
  );
  
  if (!confirmed) return;
  
  try {
    await invoke("remove_library_source", { id });
    await fetchSources();
  } catch (e) {
    console.error("Failed to remove source:", e);
    await dialog.error("删除音乐库源失败: " + String(e));
  }
}

onMounted(() => {
  fetchSources();
});
</script>

<style scoped>
.settings-view {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
}

.settings-view h1 {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 24px;
}

.settings-section {
  margin-bottom: 32px;
}

.settings-section h2 {
  font-size: 18px;
  font-weight: 500;
  margin-bottom: 16px;
  color: #94a3b8;
}

.setting-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px;
  background-color: rgba(255, 255, 255, 0.03);
  border-radius: 8px;
}

.setting-item label {
  font-size: 14px;
  color: #e2e8f0;
}

.path-input {
  display: flex;
  gap: 8px;
}

.path-input input {
  flex: 1;
  padding: 8px 12px;
  background-color: #1e293b;
  border: 1px solid #334155;
  border-radius: 6px;
  color: #e2e8f0;
  font-size: 14px;
}

.path-input button {
  padding: 8px 16px;
  background-color: #3b82f6;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
}

.path-input button:hover {
  background-color: #2563eb;
}

input[type="range"] {
  width: 100%;
}

.source-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 12px;
}

.source-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background-color: rgba(255, 255, 255, 0.03);
  border-radius: 6px;
}

.source-path {
  font-size: 14px;
  color: #e2e8f0;
  word-break: break-all;
  padding-right: 12px;
}

.remove-btn {
  padding: 4px 10px;
  background-color: #ef4444;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  white-space: nowrap;
  flex-shrink: 0;
}

.remove-btn:hover {
  background-color: #dc2626;
}

.empty-tip {
  padding: 16px;
  text-align: center;
  color: #64748b;
  font-size: 14px;
}
</style>
