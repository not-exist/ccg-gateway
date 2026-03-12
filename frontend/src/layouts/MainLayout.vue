<template>
  <el-container class="layout-container">
    <el-aside width="200px" class="sidebar">
      <div class="logo">
        <h2>CCG Gateway</h2>
      </div>
      <el-menu
        :default-active="activeMenu"
        router
        background-color="#304156"
        text-color="#bfcbd9"
        active-text-color="#409EFF"
      >
        <el-menu-item index="/">
          <el-icon><Monitor /></el-icon>
          <span>仪表盘</span>
        </el-menu-item>
        <el-menu-item index="/providers">
          <el-icon><Connection /></el-icon>
          <span>服务商管理</span>
        </el-menu-item>
        <el-menu-item index="/logs">
          <el-icon><Tickets /></el-icon>
          <span>日志管理</span>
        </el-menu-item>
        <el-menu-item index="/sessions">
          <el-icon><ChatDotRound /></el-icon>
          <span>会话管理</span>
        </el-menu-item>
        <el-menu-item index="/config">
          <el-icon><Setting /></el-icon>
          <span>全局配置</span>
        </el-menu-item>
        <el-menu-item index="/mcp">
          <el-icon><Cpu /></el-icon>
          <span>MCP 管理</span>
        </el-menu-item>
        <el-menu-item index="/prompts">
          <el-icon><Document /></el-icon>
          <span>提示词管理</span>
        </el-menu-item>
        <el-menu-item index="/skills">
          <el-icon><MagicStick /></el-icon>
          <span>Skill 管理</span>
        </el-menu-item>
      </el-menu>
      <div class="sidebar-footer">
        <span class="version">v{{ appVersion }}</span>
      </div>
    </el-aside>
    <el-container>
      <el-header class="header">
        <div class="header-content">
          <span class="page-title">{{ pageTitle }}</span>
          <div class="header-right">
            <el-button
              type="primary"
              link
              :icon="Refresh"
              @click="handleCheckUpdate"
              :loading="checkingUpdate"
            >
              检查更新
            </el-button>
            <el-button
              type="primary"
              link
              :icon="Link"
              @click="openGithubRepo"
            >
              GitHub
            </el-button>
          </div>
        </div>
      </el-header>
      <el-main class="main-content">
        <router-view />
      </el-main>
    </el-container>
  </el-container>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { getVersion } from '@tauri-apps/api/app'
import { Refresh, Link } from '@element-plus/icons-vue'
import { checkForUpdates } from '@/utils/updater'
import { open } from '@tauri-apps/plugin-shell'

const route = useRoute()

const appVersion = ref('0.0.0')
const checkingUpdate = ref(false)

const activeMenu = computed(() => route.path)

const pageTitle = computed(() => {
  const titles: Record<string, string> = {
    '/': '仪表盘',
    '/providers': '服务商管理',
    '/sessions': '会话管理',
    '/logs': '日志管理',
    '/config': '全局配置',
    '/mcp': 'MCP 管理',
    '/prompts': '提示词管理',
    '/skills': 'Skill 管理'
  }
  return titles[route.path] || 'CCG Gateway'
})

async function handleCheckUpdate() {
  checkingUpdate.value = true
  try {
    await checkForUpdates(false)
  } finally {
    checkingUpdate.value = false
  }
}

async function openGithubRepo() {
  await open('https://github.com/mos1128/ccg-gateway')
}

onMounted(async () => {
  // 获取应用版本
  appVersion.value = await getVersion()
  
  // 静默检查更新
  checkForUpdates(true)
})
</script>

<style scoped>
.layout-container {
  height: 100vh;
}

.sidebar {
  background-color: #304156;
  position: relative;
}

.logo {
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
}

.logo h2 {
  margin: 0;
  font-size: 18px;
}

.sidebar-footer {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 12px;
  text-align: center;
  border-top: 1px solid #3d4a5a;
}

.version {
  color: #8a9aad;
  font-size: 12px;
}

.header {
  background-color: #fff;
  border-bottom: 1px solid #e6e6e6;
  padding: 0 20px;
}

.header-content {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.page-title {
  font-size: 18px;
  font-weight: 500;
}

.main-content {
  background-color: #f5f7fa;
  padding: 20px;
}
</style>
