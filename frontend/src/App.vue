<template>
  <router-view />
</template>

<script setup lang="ts">
</script>

<style>
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;600&display=swap');

*, *::before, *::after {
  box-sizing: border-box;
}

html, body, #app {
  margin: 0;
  padding: 0;
  height: 100%;
  font-family: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
}

.mono {
  font-family: "JetBrains Mono", monospace !important;
}

.el-textarea__inner::-webkit-scrollbar {
  width: 14px;
}
.el-textarea__inner::-webkit-scrollbar-thumb {
  background: #c0c4cc;
  border-radius: 4px;
  border: 4px solid transparent;
  background-clip: padding-box;
}
.el-textarea__inner::-webkit-scrollbar-track {
  background: transparent;
}
</style>
