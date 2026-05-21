import { ref } from 'vue';
import type { ConfirmDialogOptions } from '../components/ConfirmDialog.vue';

const dialogRef = ref<any>(null);

export function setDialogRef(ref: any) {
  dialogRef.value = ref;
}

export function useDialog() {
  async function confirm(options: ConfirmDialogOptions): Promise<boolean | string | null> {
    if (!dialogRef.value) {
      console.error('Dialog component not mounted');
      return false;
    }
    return await dialogRef.value.show(options);
  }

  async function warning(message: string, title = '警告'): Promise<boolean> {
    const result = await confirm({
      title,
      message,
      type: 'warning',
      confirmText: '确定',
      cancelText: '取消',
    });
    return result === true;
  }

  async function error(message: string, title = '错误'): Promise<boolean> {
    const result = await confirm({
      title,
      message,
      type: 'error',
      confirmText: '知道了',
      cancelText: '取消',
    });
    return result === true;
  }

  async function info(message: string, title = '提示'): Promise<boolean> {
    const result = await confirm({
      title,
      message,
      type: 'info',
      confirmText: '确定',
      cancelText: '取消',
    });
    return result === true;
  }

  async function success(message: string, title = '成功'): Promise<boolean> {
    const result = await confirm({
      title,
      message,
      type: 'success',
      confirmText: '好的',
      cancelText: '关闭',
    });
    return result === true;
  }

  async function prompt(
    message: string,
    title = '输入',
    defaultValue = '',
    placeholder = '请输入...'
  ): Promise<string | null> {
    const result = await confirm({
      title,
      message,
      type: 'prompt',
      confirmText: '确定',
      cancelText: '取消',
      showInput: true,
      inputPlaceholder: placeholder,
      inputDefaultValue: defaultValue,
    });
    return typeof result === 'string' ? result : null;
  }

  return {
    confirm,
    warning,
    error,
    info,
    success,
    prompt,
  };
}
