<template>
  <router-view />
</template>

<script setup lang="ts">
</script>

<style>
/* ========== 字体规范 ========== */
:root {
  /* 字体权重 */
  --fw-normal: 400;
  --fw-medium: 600;
  --fw-bold: 700;

  /* 字体大小 */
  --fs-xs: 11px;
  --fs-sm: 12px;
  --fs-base: 13px;
  --fs-md: 14px;
  --fs-lg: 15px;
  --fs-xl: 16px;
  --fs-2xl: 22px;
  --fs-3xl: 28px;
}

*, *::before, *::after {
  box-sizing: border-box;
}

html, body, #app {
  margin: 0;
  padding: 0;
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  font-size: var(--fs-base);
  font-weight: var(--fw-normal);
}

/* 等宽字体 */
.mono, .code-block, code, pre {
  font-family: ui-monospace, "SF Mono", Consolas, "Liberation Mono", Menlo, monospace;
}

/* ========== 字体大小类 ========== */
.text-xs { font-size: var(--fs-xs); }
.text-sm { font-size: var(--fs-sm); }
.text-base { font-size: var(--fs-base); }
.text-md { font-size: var(--fs-md); }
.text-lg { font-size: var(--fs-lg); }
.text-xl { font-size: var(--fs-xl); }
.text-2xl { font-size: var(--fs-2xl); }
.text-3xl { font-size: var(--fs-3xl); }

/* ========== 字体权重类 ========== */
.fw-normal { font-weight: var(--fw-normal); }
.fw-medium { font-weight: var(--fw-medium); }
.fw-bold { font-weight: var(--fw-bold); }

/* ========== 语义化文本类 ========== */
.text-body { font-size: var(--fs-base); font-weight: var(--fw-normal); }
.text-label { font-size: var(--fs-sm); font-weight: var(--fw-medium); }
.text-button { font-size: var(--fs-md); font-weight: var(--fw-normal); }
.text-nav { font-size: var(--fs-md); font-weight: var(--fw-normal); }
.text-nav-active { font-size: var(--fs-md); font-weight: var(--fw-medium); }
.text-card-title { font-size: var(--fs-lg); font-weight: var(--fw-medium); }
.text-section-title { font-size: var(--fs-xl); font-weight: var(--fw-bold); }
.text-page-title { font-size: var(--fs-2xl); font-weight: var(--fw-bold); }
.text-page-title-lg { font-size: var(--fs-3xl); font-weight: var(--fw-bold); }
.text-tag { font-size: var(--fs-xs); font-weight: var(--fw-medium); }
.text-mono { font-size: var(--fs-sm); font-weight: var(--fw-normal); }
.text-mono-medium { font-size: var(--fs-sm); font-weight: var(--fw-medium); }
.text-mono-md { font-size: var(--fs-md); font-weight: var(--fw-normal); }
.text-muted { color: #94a3b8; }
.text-secondary { color: #64748b; }
.text-primary { color: #0f172a; }
.text-info { color: #0ea5e9; }
.text-upper { text-transform: uppercase; letter-spacing: 0.5px; }
.text-center { text-align: center; }

/* ========== Global Ethereal Scrollbar ========== */
::-webkit-scrollbar {
  width: 14px;
  height: 14px;
}

::-webkit-scrollbar-thumb {
  background-color: #cbd5e1;
  border-radius: 10px;
  border: 4px solid transparent;
  background-clip: padding-box;
}

::-webkit-scrollbar-thumb:hover {
  background-color: #94a3b8;
}

::-webkit-scrollbar-track {
  background-color: transparent;
}

/* Element Plus Specific Overrides */
.el-textarea__inner::-webkit-scrollbar {
  width: 14px;
}
</style>
