<template>
  <div class="config-page">
    <div class="page-header">
      <h1 class="page-title">全局设置</h1>
    </div>

    <div style="display: flex; gap: 24px; align-items: flex-start;">
      <!-- Left Column -->
      <div style="flex: 1; display: flex; flex-direction: column; gap: 24px;">
        
        <!-- 基础配置 -->
        <div class="b-card" style="margin-bottom: 0;">
          <div class="b-card-title">基础配置</div>
          <div style="margin-bottom: 16px; display: flex; align-items: center;">
            <div style="width: 120px; font-size: 14px; color: #475569; font-weight: 500;">流式首字节超时</div>
            <div style="display: flex; align-items: center; gap: 8px; flex: 1;">
              <input type="number" v-model.number="timeoutForm.stream_first_byte_timeout" class="c-input" style="flex: 1;">
              <span style="color: #94a3b8; font-size: 14px;">秒</span>
            </div>
          </div>
          <div style="margin-bottom: 16px; display: flex; align-items: center;">
            <div style="width: 120px; font-size: 14px; color: #475569; font-weight: 500;">流式空闲超时</div>
            <div style="display: flex; align-items: center; gap: 8px; flex: 1;">
              <input type="number" v-model.number="timeoutForm.stream_idle_timeout" class="c-input" style="flex: 1;">
              <span style="color: #94a3b8; font-size: 14px;">秒</span>
            </div>
          </div>
          <div style="margin-bottom: 16px; display: flex; align-items: center;">
            <div style="width: 120px; font-size: 14px; color: #475569; font-weight: 500;">非流式超时</div>
            <div style="display: flex; align-items: center; gap: 8px; flex: 1;">
              <input type="number" v-model.number="timeoutForm.non_stream_timeout" class="c-input" style="flex: 1;">
              <span style="color: #94a3b8; font-size: 14px;">秒</span>
            </div>
          </div>
          <div style="margin-top: 24px; display: flex; justify-content: flex-end;">
            <button class="b-button" @click="saveTimeouts">保存</button>
          </div>
        </div>

        <!-- 备份与恢复 -->
        <div class="b-card" style="margin-bottom: 0;">
          <div class="b-card-title">备份与恢复</div>
          <div class="b-segmented" style="width: 100%; margin-bottom: 20px;">
            <div class="b-seg-btn" :class="{ active: activeBackupTab === 'local' }" @click="activeBackupTab = 'local'" style="flex: 1;">本地备份</div>
            <div class="b-seg-btn" :class="{ active: activeBackupTab === 'webdav' }" @click="activeBackupTab = 'webdav'" style="flex: 1;">WebDAV</div>
          </div>

          <div v-if="activeBackupTab === 'local'">
            <p style="font-size: 13px; color: #64748b; margin-bottom: 20px; line-height: 1.5;">
              将数据库文件导出到本地，或从本地文件恢复
            </p>
            <div style="display: flex; gap: 12px; justify-content: flex-end;">
              <button class="b-button" style="background:#10b981; box-shadow: 0 2px 4px rgba(16,185,129,0.2);" @click="handleExportLocal" :disabled="exportingLocal">
                {{ exportingLocal ? '导出中...' : '导出' }}
              </button>
              <el-upload :show-file-list="false" :before-upload="handleImportLocal" accept=".db" style="display: inline-block;">
                 <button class="b-button" :disabled="importingLocal" style="background:#f59e0b; box-shadow: 0 2px 4px rgba(245,158,11,0.2);">
                   {{ importingLocal ? '导入中...' : '导入' }}
                 </button>
              </el-upload>
            </div>
          </div>

          <div v-if="activeBackupTab === 'webdav'">
            <div style="margin-bottom: 12px; display: flex; align-items: center;">
              <div style="width: 80px; font-size: 14px; color: #475569; font-weight: 500;">服务器</div>
              <input type="text" v-model="webdavForm.url" placeholder="https://dav.example.com" class="c-input" style="flex: 1;">
            </div>
            <div style="margin-bottom: 12px; display: flex; align-items: center;">
              <div style="width: 80px; font-size: 14px; color: #475569; font-weight: 500;">用户名</div>
              <input type="text" v-model="webdavForm.username" class="c-input" style="flex: 1;">
            </div>
            <div style="margin-bottom: 24px; display: flex; align-items: center;">
              <div style="width: 80px; font-size: 14px; color: #475569; font-weight: 500;">密码</div>
              <input type="password" v-model="webdavForm.password" class="c-input" style="flex: 1;">
            </div>
            <div style="display: flex; gap: 10px; flex-wrap: wrap; justify-content: flex-end;">
              <button class="b-button-outline" @click="handleTestWebdav" :disabled="testingWebdav">
                {{ testingWebdav ? '测试中...' : '测试链接' }}
              </button>
              <button class="b-button-outline" @click="handleSaveWebdav" :disabled="savingWebdav">保存配置</button>
              <button class="b-button" style="background:#10b981; box-shadow: 0 2px 4px rgba(16,185,129,0.2);" @click="handleExportWebdav" :disabled="exportingWebdav">
                {{ exportingWebdav ? '导出中...' : '导出' }}
              </button>
              <button class="b-button" style="background:#f59e0b; box-shadow: 0 2px 4px rgba(245,158,11,0.2);" @click="handleShowWebdavList" :disabled="loadingWebdavList">
                {{ loadingWebdavList ? '加载中...' : '导入' }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Right Column -->
      <div style="flex: 1; display: flex; flex-direction: column;">
        <div class="b-card" style="flex: 1; margin-bottom: 0;">
          <div class="b-card-title">CLI全局配置</div>
          
          <div class="b-segmented" style="width: 100%; margin-bottom: 24px;">
            <div class="b-seg-btn" :class="{ active: activeCliTab === 'claude_code' }" @click="activeCliTab = 'claude_code'" style="flex: 1;">ClaudeCode</div>
            <div class="b-seg-btn" :class="{ active: activeCliTab === 'codex' }" @click="activeCliTab = 'codex'" style="flex: 1;">Codex</div>
            <div class="b-seg-btn" :class="{ active: activeCliTab === 'gemini' }" @click="activeCliTab = 'gemini'" style="flex: 1;">Gemini</div>
          </div>

          <div v-if="activeCliTab === 'claude_code'">
             <CliSettingsForm cli-type="claude_code" :settings="settingsStore.settings?.cli_settings?.claude_code" @save="saveCli" />
          </div>
          <div v-if="activeCliTab === 'codex'">
             <CliSettingsForm cli-type="codex" :settings="settingsStore.settings?.cli_settings?.codex" @save="saveCli" />
          </div>
          <div v-if="activeCliTab === 'gemini'">
             <CliSettingsForm cli-type="gemini" :settings="settingsStore.settings?.cli_settings?.gemini" @save="saveCli" />
          </div>
        </div>
      </div>
    </div>

    <!-- WebDAV Backup List Dialog remains using Element Plus dialog as it works perfectly for data grids -->
    <el-dialog v-model="webdavListVisible" title="选择备份文件" width="700px">
      <el-table :data="webdavBackups" v-loading="loadingWebdavList">
        <el-table-column prop="filename" label="文件名" min-width="280" />
        <el-table-column prop="size" label="大小" width="100">
          <template #default="{ row }">{{ formatSize(row.size) }}</template>
        </el-table-column>
        <el-table-column label="操作" width="160">
          <template #default="{ row }">
            <el-button type="primary" size="small" @click="handleImportWebdav(row.filename)" :loading="importingWebdav">导入</el-button>
            <el-button type="danger" size="small" @click="handleDeleteWebdav(row.filename)" :loading="deletingWebdav">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, computed } from 'vue'
import { ElMessageBox } from 'element-plus'
import { notify } from '@/utils/notification'
import { useSettingsStore } from '@/stores/settings'
import { useUiStore } from '@/stores/ui'
import CliSettingsForm from './components/CliSettingsForm.vue'
import * as backupApi from '@/api/backup'
import type { WebdavSettings, WebdavBackup } from '@/api/backup'

const settingsStore = useSettingsStore()
const uiStore = useUiStore()

const activeCliTab = computed({
  get: () => uiStore.configActiveCliTab,
  set: (val) => uiStore.setConfigActiveCliTab(val as 'claude_code' | 'codex' | 'gemini')
})
const activeBackupTab = computed({
  get: () => uiStore.configActiveBackupTab,
  set: (val) => uiStore.setConfigActiveBackupTab(val as 'local' | 'webdav')
})

const timeoutForm = ref({
  stream_first_byte_timeout: 30,
  stream_idle_timeout: 60,
  non_stream_timeout: 120
})

watch(() => settingsStore.settings, (settings) => {
  if (settings) {
    timeoutForm.value = { ...settings.timeouts }
  }
}, { immediate: true })

async function saveTimeouts() {
  await settingsStore.updateTimeouts(timeoutForm.value)
  notify('超时配置已保存')
}

async function saveCli(cliType: string, data: any) {
  await settingsStore.updateCli(cliType, data)
  notify('CLI 配置已保存')
}

// Backup related
const webdavForm = ref<WebdavSettings>({ url: '', username: '', password: '' })
const exportingLocal = ref(false)
const importingLocal = ref(false)
const testingWebdav = ref(false)
const savingWebdav = ref(false)
const exportingWebdav = ref(false)
const loadingWebdavList = ref(false)
const importingWebdav = ref(false)
const deletingWebdav = ref(false)
const webdavListVisible = ref(false)
const webdavBackups = ref<WebdavBackup[]>([])

async function loadWebdavSettings() {
  try {
    const { data } = await backupApi.getWebdavSettings()
    webdavForm.value = data
  } catch {}
}

async function handleExportLocal() {
  exportingLocal.value = true
  try {
    const { data } = await backupApi.exportToLocal()
    const url = window.URL.createObjectURL(new Blob([data]))
    const link = document.createElement('a')
    link.href = url
    link.download = `ccg_gateway_${new Date().toISOString().slice(0, 10)}.db`
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    window.URL.revokeObjectURL(url)
    notify('导出成功（默认保存至下载文件夹）')
  } catch (error: any) {
    notify(error?.message || '导出失败', 'error')
  } finally {
    exportingLocal.value = false
  }
}

async function handleImportLocal(file: File) {
  await ElMessageBox.confirm('导入将覆盖当前所有数据，确定继续？', '警告', { type: 'warning' })
  importingLocal.value = true
  try {
    await backupApi.importFromLocal(file)
    notify('导入成功，应用将自动退出，请重新打开应用')
  } finally {
    importingLocal.value = false
  }
  return false
}

async function handleTestWebdav() {
  testingWebdav.value = true
  try {
    const { data } = await backupApi.testWebdavConnection(webdavForm.value)
    if (data.success) {
      notify('连接成功')
    } else {
      notify('连接失败', 'error')
    }
  } catch (error: any) {
    notify(error?.message || '连接失败', 'error')
  } finally {
    testingWebdav.value = false
  }
}

async function handleSaveWebdav() {
  savingWebdav.value = true
  try {
    await backupApi.updateWebdavSettings(webdavForm.value)
    notify('WebDAV 配置已保存')
  } catch (error: any) {
    notify(error?.message || '保存失败', 'error')
  } finally {
    savingWebdav.value = false
  }
}

async function handleExportWebdav() {
  exportingWebdav.value = true
  try {
    const { data } = await backupApi.exportToWebdav()
    notify(`导出成功: ${data.filename}`)
  } catch (error: any) {
    notify(error?.message || '导出失败', 'error')
  } finally {
    exportingWebdav.value = false
  }
}

async function handleShowWebdavList() {
  webdavListVisible.value = true
  loadingWebdavList.value = true
  try {
    const { data } = await backupApi.listWebdavBackups()
    webdavBackups.value = data.backups
  } finally {
    loadingWebdavList.value = false
  }
}

async function handleImportWebdav(filename: string) {
  await ElMessageBox.confirm('导入将覆盖当前所有数据，确定继续？', '警告', { type: 'warning' })
  importingWebdav.value = true
  try {
    await backupApi.importFromWebdav(filename)
    notify('导入成功，应用将自动退出，请重新打开应用')
    webdavListVisible.value = false
  } catch (error: any) {
    notify(error?.message || '导入失败', 'error')
  } finally {
    importingWebdav.value = false
  }
}

async function handleDeleteWebdav(filename: string) {
  await ElMessageBox.confirm(`确定要删除远程备份 ${filename} 吗？`, '警告', { type: 'warning' })
  deletingWebdav.value = true
  try {
    await backupApi.deleteWebdavBackup(filename)
    notify('删除成功')
    await handleShowWebdavList()
  } catch (error: any) {
    notify(error?.message || '删除失败', 'error')
  } finally {
    deletingWebdav.value = false
  }
}

function formatSize(bytes: number) {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / 1024 / 1024).toFixed(1) + ' MB'
}

onMounted(() => {
  settingsStore.fetchSettings()
  loadWebdavSettings()
})
</script>

<style scoped>
.page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 32px; }
.page-title { font-size: 28px; font-weight: 700; margin: 0; letter-spacing: -0.5px; }

.b-card { background: #ffffff; border-radius: 16px; box-shadow: 0 4px 12px rgba(0,0,0,0.03); padding: 24px; margin-bottom: 24px; transition: transform 0.2s, box-shadow 0.2s; }
.b-card:hover { box-shadow: 0 8px 24px rgba(0,0,0,0.06); transform: translateY(-2px); }
.b-card-title { font-size: 16px; font-weight: 600; margin-bottom: 20px; }

.b-segmented { display: inline-flex; background: #e2e8f0; padding: 4px; border-radius: 10px; }
.b-seg-btn { text-align: center; padding: 6px 16px; font-size: 14px; color: #475569; border-radius: 8px; cursor: pointer; font-weight: 500; transition: all 0.2s ease; }
.b-seg-btn.active { background: #ffffff; color: #0ea5e9; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
.b-seg-btn:not(.active):hover { color: #0f172a; }

.b-button { background: #0ea5e9; color: white; border: none; padding: 8px 16px; border-radius: 8px; font-size: 14px; font-weight: 500; cursor: pointer; transition: transform 0.2s, box-shadow 0.2s; outline: none; }
.b-button:hover:not(:disabled) { transform: translateY(-1px); box-shadow: 0 4px 8px rgba(14, 165, 233, 0.3); }
.b-button:disabled { opacity: 0.6; cursor: not-allowed; }

.b-button-outline { background: white; color: #0f172a; border: 1px solid #e2e8f0; padding: 8px 16px; border-radius: 8px; font-size: 14px; font-weight: 500; cursor: pointer; transition: background 0.2s; outline: none; }
.b-button-outline:hover:not(:disabled) { background: #f8fafc; }
.b-button-outline:disabled { opacity: 0.6; cursor: not-allowed; }

.c-input { width: 100%; padding: 10px 14px; border: 1px solid #e2e8f0; border-radius: 8px; font-size: 14px; outline: none; transition: border-color 0.2s, box-shadow 0.2s; }
.c-input:focus { border-color: #0ea5e9; box-shadow: 0 0 0 3px rgba(14, 165, 233, 0.1); }
</style>
