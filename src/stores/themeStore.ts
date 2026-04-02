import { defineStore } from 'pinia';
import type { Theme } from '../types';

export const useThemeStore = defineStore('theme', {
  state: () => ({
    currentTheme: 'mint' as string,
    themes: [
      { id: 'mint', name: '薄荷绿', color: '#4ADE80', bg: 'bg-emerald-50', text: 'text-emerald-600', btn: 'bg-emerald-500', shadow: 'shadow-emerald-200' },
      { id: 'sakura', name: '樱花粉', color: '#F472B6', bg: 'bg-pink-50', text: 'text-pink-600', btn: 'bg-pink-500', shadow: 'shadow-pink-200' },
      { id: 'lavender', name: '薰衣草', color: '#A78BFA', bg: 'bg-violet-50', text: 'text-violet-600', btn: 'bg-violet-500', shadow: 'shadow-violet-200' },
      { id: 'sky', name: '晴空蓝', color: '#60A5FA', bg: 'bg-blue-50', text: 'text-blue-600', btn: 'bg-blue-500', shadow: 'shadow-blue-200' }
    ] as Theme[]
  }),
  actions: {
    // 设置主题
    setTheme(themeId: string) {
      this.currentTheme = themeId;
    },
    // 获取当前主题
    getCurrentTheme() {
      return this.themes.find(t => t.id === this.currentTheme);
    }
  }
});
