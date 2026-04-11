import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export type Theme = 'light' | 'dark'

export const useThemeStore = defineStore('theme', () => {
  const theme = ref<Theme>((localStorage.getItem('theme') as Theme) || 'light')

  function applyTheme(t: Theme) {
    const html = document.documentElement
    if (t === 'dark') {
      html.classList.add('dark')
    } else {
      html.classList.remove('dark')
    }
    localStorage.setItem('theme', t)
  }

  function toggleTheme() {
    theme.value = theme.value === 'light' ? 'dark' : 'light'
  }

  // 初始化应用主题
  applyTheme(theme.value)

  // 监听变化自动应用
  watch(theme, (t) => applyTheme(t))

  return { theme, toggleTheme }
})