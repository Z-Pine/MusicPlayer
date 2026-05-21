<template>
  <Teleport to="body">
    <Transition name="dialog-fade">
      <div v-if="visible" class="dialog-overlay" @click="handleOverlayClick">
        <Transition name="dialog-scale">
          <div v-if="visible" class="dialog-container" @click.stop>
            <div class="dialog-icon" :class="iconClass">
              <span class="icon-emoji">{{ iconEmoji }}</span>
            </div>
            <h3 class="dialog-title">{{ title }}</h3>
            <p class="dialog-message" v-if="message">{{ message }}</p>
            
            <!-- 输入框（用于 prompt 类型） -->
            <input
              v-if="showInput"
              ref="inputRef"
              v-model="inputValue"
              type="text"
              class="dialog-input"
              :placeholder="inputPlaceholder"
              @keydown.enter="handleConfirm"
              @keydown.esc="handleCancel"
            />
            
            <div class="dialog-actions">
              <button class="dialog-btn cancel-btn" @click="handleCancel">
                <span class="btn-icon">✕</span>
                <span>{{ cancelText }}</span>
                <span class="btn-hint">ESC</span>
              </button>
              <button class="dialog-btn confirm-btn" @click="handleConfirm">
                <span class="btn-icon">✓</span>
                <span>{{ confirmText }}</span>
                <span class="btn-hint">Enter</span>
              </button>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, onUnmounted, nextTick } from 'vue';

export interface ConfirmDialogOptions {
  title?: string;
  message?: string;
  confirmText?: string;
  cancelText?: string;
  type?: 'warning' | 'error' | 'info' | 'success' | 'prompt';
  showInput?: boolean;
  inputPlaceholder?: string;
  inputDefaultValue?: string;
}

const visible = ref(false);
const title = ref('确认操作');
const message = ref('');
const confirmText = ref('确定');
const cancelText = ref('取消');
const type = ref<'warning' | 'error' | 'info' | 'success' | 'prompt'>('warning');
const showInput = ref(false);
const inputPlaceholder = ref('');
const inputValue = ref('');
const inputRef = ref<HTMLInputElement | null>(null);

let resolvePromise: ((value: boolean | string | null) => void) | null = null;

const iconEmoji = computed(() => {
  switch (type.value) {
    case 'warning':
      return '⚠️';
    case 'error':
      return '❌';
    case 'info':
    case 'prompt':
      return 'ℹ️';
    case 'success':
      return '✓';
    default:
      return '⚠️';
  }
});

const iconClass = computed(() => {
  if (type.value === 'prompt') return 'icon-info';
  return `icon-${type.value}`;
});

function show(options: ConfirmDialogOptions): Promise<boolean | string | null> {
  title.value = options.title || '确认操作';
  message.value = options.message || '';
  confirmText.value = options.confirmText || '确定';
  cancelText.value = options.cancelText || '取消';
  type.value = options.type || 'warning';
  showInput.value = options.showInput || false;
  inputPlaceholder.value = options.inputPlaceholder || '';
  inputValue.value = options.inputDefaultValue || '';
  visible.value = true;

  // 如果显示输入框，聚焦到输入框
  if (showInput.value) {
    nextTick(() => {
      inputRef.value?.focus();
      inputRef.value?.select();
    });
  }

  return new Promise((resolve) => {
    resolvePromise = resolve;
  });
}

function handleConfirm() {
  visible.value = false;
  if (resolvePromise) {
    if (showInput.value) {
      // 如果是输入框模式，返回输入的值
      resolvePromise(inputValue.value.trim() || null);
    } else {
      // 否则返回 true
      resolvePromise(true);
    }
    resolvePromise = null;
  }
}

function handleCancel() {
  visible.value = false;
  if (resolvePromise) {
    if (showInput.value) {
      // 如果是输入框模式，返回 null
      resolvePromise(null);
    } else {
      // 否则返回 false
      resolvePromise(false);
    }
    resolvePromise = null;
  }
}

function handleOverlayClick() {
  handleCancel();
}

function handleKeydown(e: KeyboardEvent) {
  if (!visible.value) return;
  
  // 如果显示输入框，Enter 和 ESC 由输入框自己处理
  if (showInput.value) return;
  
  if (e.key === 'Escape') {
    e.preventDefault();
    handleCancel();
  } else if (e.key === 'Enter') {
    e.preventDefault();
    handleConfirm();
  }
}

// 监听对话框显示状态，添加/移除键盘事件监听
watch(visible, (newVisible) => {
  if (newVisible) {
    document.addEventListener('keydown', handleKeydown);
  } else {
    document.removeEventListener('keydown', handleKeydown);
  }
});

// 组件卸载时清理事件监听
onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown);
});

defineExpose({
  show,
});
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.dialog-container {
  background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
  border-radius: 16px;
  padding: 32px;
  width: 420px;
  max-width: 90vw;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  border: 1px solid rgba(59, 130, 246, 0.2);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}

.dialog-icon {
  width: 72px;
  height: 72px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 36px;
  animation: iconBounce 0.5s ease-out;
}

.icon-warning {
  background: linear-gradient(135deg, rgba(251, 191, 36, 0.15) 0%, rgba(245, 158, 11, 0.15) 100%);
  border: 2px solid rgba(251, 191, 36, 0.4);
  box-shadow: 0 0 20px rgba(251, 191, 36, 0.2);
}

.icon-error {
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.15) 0%, rgba(220, 38, 38, 0.15) 100%);
  border: 2px solid rgba(239, 68, 68, 0.4);
  box-shadow: 0 0 20px rgba(239, 68, 68, 0.2);
}

.icon-info {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.15) 0%, rgba(37, 99, 235, 0.15) 100%);
  border: 2px solid rgba(59, 130, 246, 0.4);
  box-shadow: 0 0 20px rgba(59, 130, 246, 0.2);
}

.icon-success {
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.15) 0%, rgba(5, 150, 105, 0.15) 100%);
  border: 2px solid rgba(16, 185, 129, 0.4);
  box-shadow: 0 0 20px rgba(16, 185, 129, 0.2);
}

.icon-emoji {
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.3));
}

.dialog-title {
  font-size: 22px;
  font-weight: 700;
  color: #f8fafc;
  margin: 0;
  text-align: center;
}

.dialog-message {
  font-size: 15px;
  color: #cbd5e1;
  line-height: 1.6;
  margin: 0;
  text-align: center;
  white-space: pre-wrap;
}

.dialog-input {
  width: 100%;
  padding: 12px 16px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(59, 130, 246, 0.3);
  border-radius: 8px;
  color: #e2e8f0;
  font-size: 15px;
  outline: none;
  transition: all 0.2s;
}

.dialog-input:focus {
  border-color: rgba(59, 130, 246, 0.6);
  background: rgba(255, 255, 255, 0.08);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.dialog-input::placeholder {
  color: #64748b;
}

.dialog-actions {
  display: flex;
  gap: 12px;
  width: 100%;
  margin-top: 8px;
}

.dialog-btn {
  flex: 1;
  padding: 12px 24px;
  border: none;
  border-radius: 10px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.cancel-btn {
  background: rgba(255, 255, 255, 0.08);
  color: #cbd5e1;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.cancel-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  border-color: rgba(255, 255, 255, 0.2);
  transform: translateY(-1px);
}

.cancel-btn:active {
  transform: translateY(0);
}

.confirm-btn {
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  color: white;
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
}

.confirm-btn:hover {
  background: linear-gradient(135deg, #2563eb 0%, #1d4ed8 100%);
  box-shadow: 0 6px 16px rgba(59, 130, 246, 0.4);
  transform: translateY(-1px);
}

.confirm-btn:active {
  transform: translateY(0);
}

.btn-icon {
  font-size: 16px;
}

.btn-hint {
  font-size: 11px;
  opacity: 0.6;
  padding: 2px 6px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 4px;
  font-family: monospace;
  margin-left: 4px;
}

/* 动画 */
.dialog-fade-enter-active,
.dialog-fade-leave-active {
  transition: opacity 0.3s ease;
}

.dialog-fade-enter-from,
.dialog-fade-leave-to {
  opacity: 0;
}

.dialog-scale-enter-active {
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.dialog-scale-leave-active {
  transition: all 0.2s ease-out;
}

.dialog-scale-enter-from {
  opacity: 0;
  transform: scale(0.8) translateY(-20px);
}

.dialog-scale-leave-to {
  opacity: 0;
  transform: scale(0.9);
}

@keyframes iconBounce {
  0% {
    transform: scale(0);
    opacity: 0;
  }
  50% {
    transform: scale(1.1);
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
}
</style>
