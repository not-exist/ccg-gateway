<template>
  <div class="plugins-page">
    <el-tabs v-model="activeTab">
      <!-- 插件列表 -->
      <el-tab-pane label="插件" name="plugins">
        <div class="page-header">
          <el-input
            v-model="pluginSearchQuery"
            placeholder="搜索插件..."
            clearable
            style="width: 250px"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
          <el-button type="primary" @click="handleRefresh" :loading="loading">
            刷新列表
          </el-button>
        </div>
        <el-table :data="filteredPlugins" stripe style="width: 100%" v-loading="loading">
          <el-table-column label="名称" min-width="150">
            <template #default="{ row }">
              <div class="plugin-name-cell">
                <span>{{ row.name }}</span>
                <span v-if="row.version" class="plugin-meta">v{{ row.version }}</span>
                <span class="plugin-meta">@{{ row.marketplace_name }}</span>
              </div>
            </template>
          </el-table-column>
          <el-table-column prop="description" label="描述" min-width="200">
            <template #default="{ row }">
              <el-tooltip
                v-if="row.description"
                :content="row.description"
                placement="top"
                :show-after="300"
                :hide-after="0"
                popper-class="plugin-desc-tooltip"
              >
                <div class="plugin-desc">{{ row.description }}</div>
              </el-tooltip>
              <span v-else class="text-muted">-</span>
            </template>
          </el-table-column>
          <el-table-column label="状态" width="100">
            <template #default="{ row }">
              <el-tag v-if="row.is_installed" :type="row.is_enabled ? 'success' : 'info'" size="small">
                {{ row.is_enabled ? '已启用' : '已停用' }}
              </el-tag>
              <el-tag v-else type="warning" size="small">未安装</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="300">
            <template #default="{ row }">
              <div class="action-buttons">
                <template v-if="row.is_installed">
                  <el-button
                    v-if="row.is_enabled"
                    size="small"
                    :loading="operatingPlugin === row.name && operatingType === 'disable'"
                    :disabled="!!operatingPlugin"
                    @click="handleDisable(row)"
                  >停用</el-button>
                  <el-button
                    v-else
                    size="small"
                    type="primary"
                    :loading="operatingPlugin === row.name && operatingType === 'enable'"
                    :disabled="!!operatingPlugin"
                    @click="handleEnable(row)"
                  >启用</el-button>
                  <el-button
                    size="small"
                    type="warning"
                    :loading="operatingPlugin === row.name && operatingType === 'update'"
                    :disabled="!!operatingPlugin"
                    @click="handleUpdate(row)"
                  >更新</el-button>
                  <el-button
                    size="small"
                    type="danger"
                    :loading="operatingPlugin === row.name && operatingType === 'uninstall'"
                    :disabled="!!operatingPlugin"
                    @click="handleUninstall(row)"
                  >卸载</el-button>
                </template>
                <template v-else>
                  <el-button
                    size="small"
                    type="primary"
                    :loading="operatingPlugin === row.name && operatingType === 'install'"
                    :disabled="!!operatingPlugin"
                    @click="handleInstall(row)"
                  >安装</el-button>
                </template>
                <el-button
                  v-if="!favoriteIds.has(getPluginId(row))"
                  size="small"
                  :icon="Star"
                  :loading="operatingPlugin === row.name && operatingType === 'favorite'"
                  :disabled="!!operatingPlugin"
                  @click="handleAddFavorite(row)"
                />
                <el-button
                  v-else
                  size="small"
                  type="warning"
                  :icon="StarFilled"
                  :loading="operatingPlugin === row.name && operatingType === 'unfavorite'"
                  :disabled="!!operatingPlugin"
                  @click="handleRemoveFavorite(row)"
                />
              </div>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>

      <!-- 市场管理 -->
      <el-tab-pane label="市场" name="marketplaces">
        <div class="page-header">
          <span></span>
          <el-button type="primary" @click="showAddMarketDialog = true">
            <el-icon><Plus /></el-icon>
            添加市场
          </el-button>
        </div>
        <el-table :data="marketplaceList" stripe style="width: 100%" v-loading="loadingMarketplaces">
          <el-table-column prop="name" label="名称" min-width="150" />
          <el-table-column label="市场地址" min-width="250">
            <template #default="{ row }">
              <span v-if="row.marketplace_source">{{ row.marketplace_source }}</span>
              <span v-else class="text-muted">-</span>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="160">
            <template #default="{ row }">
              <el-button
                size="small"
                type="primary"
                :loading="operatingMarket === row.name && operatingMarketType === 'update'"
                :disabled="!!operatingMarket"
                @click="handleUpdateMarketplace(row)"
              >更新</el-button>
              <el-button
                size="small"
                type="danger"
                :loading="operatingMarket === row.name && operatingMarketType === 'remove'"
                :disabled="!!operatingMarket"
                @click="handleRemoveMarketplace(row)"
              >删除</el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>

      <!-- 收藏列表 -->
      <el-tab-pane label="收藏" name="favorites">
        <div class="favorites-tip">收藏的插件会跟随备份跨设备同步</div>
        <el-table :data="favoriteList" stripe style="width: 100%">
          <el-table-column label="插件名称" min-width="150">
            <template #default="{ row }">
              {{ row.plugin_name }}
            </template>
          </el-table-column>
          <el-table-column label="市场名称" min-width="150">
            <template #default="{ row }">
              {{ row.marketplace_name }}
            </template>
          </el-table-column>
          <el-table-column label="市场地址" min-width="200">
            <template #default="{ row }">
              <span v-if="row.marketplace_source">{{ row.marketplace_source }}</span>
              <span v-else class="text-muted">-</span>
            </template>
          </el-table-column>
          <el-table-column label="状态" width="100">
            <template #default="{ row }">
              <el-tag :type="row.is_installed ? 'success' : 'info'" size="small">
                {{ row.is_installed ? '已安装' : '未安装' }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="160">
            <template #default="{ row }">
              <template v-if="!row.is_installed">
                <el-button
                  size="small"
                  type="primary"
                  :loading="operatingPlugin === row.plugin_id && operatingType === 'install'"
                  :disabled="!!operatingPlugin"
                  @click="handleInstallFavorite(row)"
                >安装</el-button>
              </template>
              <el-button
                size="small"
                type="danger"
                :loading="operatingPlugin === row.plugin_id && operatingType === 'unfavorite'"
                :disabled="!!operatingPlugin"
                @click="handleRemoveFavoriteById(row)"
              >移除</el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>
    </el-tabs>

    <!-- 添加市场对话框 -->
    <el-dialog v-model="showAddMarketDialog" title="添加市场" width="500px">
      <el-form :model="marketForm" label-width="80px">
        <el-form-item label="市场地址" required>
          <el-input v-model="marketForm.url" placeholder="支持URL地址、owner/repo、本地路径" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAddMarketDialog = false" :disabled="savingMarket">取消</el-button>
        <el-button type="primary" @click="handleAddMarketplace" :loading="savingMarket">添加</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { ElMessageBox, ElNotification } from 'element-plus'
import { Plus, Search, Star, StarFilled } from '@element-plus/icons-vue'
import { pluginsApi } from '@/api/plugins'
import type { MarketplaceInfo, PluginItem, PluginFavoriteItem } from '@/types/models'

const activeTab = ref('plugins')

// 插件列表
const allPlugins = ref<PluginItem[]>([])
const loading = ref(false)
const pluginSearchQuery = ref('')

// 市场列表
const marketplaceList = ref<MarketplaceInfo[]>([])
const loadingMarketplaces = ref(false)
const showAddMarketDialog = ref(false)
const marketForm = ref({ url: '' })
const savingMarket = ref(false)

// 收藏列表（独立数据）
const favoriteList = ref<PluginFavoriteItem[]>([])

// 操作状态
const operatingPlugin = ref<string | null>(null)
const operatingType = ref<string | null>(null)

// 市场操作状态
const operatingMarket = ref<string | null>(null)
const operatingMarketType = ref<string | null>(null)

// 已安装排顶部的排序列表
const sortedPlugins = computed(() => {
  const installed = allPlugins.value.filter(p => p.is_installed)
  const notInstalled = allPlugins.value.filter(p => !p.is_installed)
  return [...installed, ...notInstalled]
})

const filteredPlugins = computed(() => {
  if (!pluginSearchQuery.value) return sortedPlugins.value
  const query = pluginSearchQuery.value.toLowerCase()
  return sortedPlugins.value.filter(p =>
    p.name.toLowerCase().includes(query) ||
    p.description?.toLowerCase().includes(query) ||
    p.marketplace_name.toLowerCase().includes(query)
  )
})

// 收藏 ID 集合（用于插件列表显示收藏状态）
const favoriteIds = computed(() => {
  return new Set(favoriteList.value.map(f => f.plugin_id))
})

// 获取 plugin_id
function getPluginId(plugin: PluginItem): string {
  return plugin.marketplace_name ? `${plugin.name}@${plugin.marketplace_name}` : plugin.name
}

// 展示 CLI 输出
function showCliOutput(output: string, isError: boolean = false) {
  if (!output) return
  ElNotification({
    title: isError ? '操作失败' : '操作结果',
    message: output.replace(/\n/g, '<br/>'),
    type: isError ? 'error' : 'success',
    duration: 5000,
    position: 'top-right',
    dangerouslyUseHTMLString: true
  })
}

// 加载所有数据
async function loadAll() {
  loading.value = true
  loadingMarketplaces.value = true
  try {
    const [plugins, marketplaces, favorites] = await Promise.all([
      pluginsApi.getAll(),
      pluginsApi.getMarketplaces(),
      pluginsApi.getFavorites()
    ])
    allPlugins.value = plugins
    marketplaceList.value = marketplaces
    favoriteList.value = favorites
  } catch (error: any) {
    const msg = error?.message || String(error) || '加载失败'
    ElNotification({
      title: '加载失败',
      message: msg,
      type: 'error',
      duration: 5000,
      position: 'top-right'
    })
  } finally {
    loading.value = false
    loadingMarketplaces.value = false
  }
}

// 刷新
async function handleRefresh() {
  loading.value = true
  try {
    allPlugins.value = await pluginsApi.refresh()
    favoriteList.value = await pluginsApi.getFavorites()
    showCliOutput('刷新成功')
  } catch (error: any) {
    showCliOutput(error?.message || '刷新失败', true)
  } finally {
    loading.value = false
  }
}

// 安装插件
async function handleInstall(plugin: PluginItem) {
  const pluginId = getPluginId(plugin)
  operatingPlugin.value = plugin.name
  operatingType.value = 'install'
  try {
    const result = await pluginsApi.install(pluginId)
    allPlugins.value = result.plugins
    favoriteList.value = await pluginsApi.getFavorites()
    showCliOutput(result.cli_output)
  } catch (error: any) {
    showCliOutput(error?.message || '安装失败', true)
  } finally {
    operatingPlugin.value = null
    operatingType.value = null
  }
}

// 卸载插件
async function handleUninstall(plugin: PluginItem) {
  const pluginId = getPluginId(plugin)
  try {
    await ElMessageBox.confirm(`确定卸载插件 "${plugin.name}"?`, '确认')
    operatingPlugin.value = plugin.name
    operatingType.value = 'uninstall'
    const result = await pluginsApi.uninstall(pluginId)
    allPlugins.value = result.plugins
    favoriteList.value = await pluginsApi.getFavorites()
    showCliOutput(result.cli_output)
  } catch (error: any) {
    if (error !== 'cancel') {
      showCliOutput(error?.message || '卸载失败', true)
    }
  } finally {
    operatingPlugin.value = null
    operatingType.value = null
  }
}

// 启用插件
async function handleEnable(plugin: PluginItem) {
  const pluginId = getPluginId(plugin)
  operatingPlugin.value = plugin.name
  operatingType.value = 'enable'
  try {
    const result = await pluginsApi.enable(pluginId)
    allPlugins.value = result.plugins
    showCliOutput(result.cli_output)
  } catch (error: any) {
    showCliOutput(error?.message || '操作失败', true)
  } finally {
    operatingPlugin.value = null
    operatingType.value = null
  }
}

// 停用插件
async function handleDisable(plugin: PluginItem) {
  const pluginId = getPluginId(plugin)
  operatingPlugin.value = plugin.name
  operatingType.value = 'disable'
  try {
    const result = await pluginsApi.disable(pluginId)
    allPlugins.value = result.plugins
    showCliOutput(result.cli_output)
  } catch (error: any) {
    showCliOutput(error?.message || '操作失败', true)
  } finally {
    operatingPlugin.value = null
    operatingType.value = null
  }
}

// 更新插件
async function handleUpdate(plugin: PluginItem) {
  const pluginId = getPluginId(plugin)
  operatingPlugin.value = plugin.name
  operatingType.value = 'update'
  try {
    const result = await pluginsApi.update(pluginId)
    allPlugins.value = result.plugins
    showCliOutput(result.cli_output)
  } catch (error: any) {
    showCliOutput(error?.message || '更新失败', true)
  } finally {
    operatingPlugin.value = null
    operatingType.value = null
  }
}

// 添加收藏
async function handleAddFavorite(plugin: PluginItem) {
  const pluginId = getPluginId(plugin)
  operatingPlugin.value = plugin.name
  operatingType.value = 'favorite'
  try {
    const result = await pluginsApi.addFavorite(pluginId, plugin.name, plugin.marketplace_name)
    allPlugins.value = result.plugins
    favoriteList.value = await pluginsApi.getFavorites()
    if (result.cli_output) {
      showCliOutput(result.cli_output)
    } else {
      showCliOutput('已收藏')
    }
  } catch (error: any) {
    showCliOutput(error?.message || '操作失败', true)
  } finally {
    operatingPlugin.value = null
    operatingType.value = null
  }
}

// 移除收藏
async function handleRemoveFavorite(plugin: PluginItem) {
  const pluginId = getPluginId(plugin)
  operatingPlugin.value = plugin.name
  operatingType.value = 'unfavorite'
  try {
    const result = await pluginsApi.removeFavorite(pluginId)
    allPlugins.value = result.plugins
    favoriteList.value = await pluginsApi.getFavorites()
    showCliOutput('已取消收藏')
  } catch (error: any) {
    showCliOutput(error?.message || '操作失败', true)
  } finally {
    operatingPlugin.value = null
    operatingType.value = null
  }
}

// 安装收藏中的插件
async function handleInstallFavorite(favorite: PluginFavoriteItem) {
  operatingPlugin.value = favorite.plugin_id
  operatingType.value = 'install'
  try {
    // 检查市场是否存在
    const exists = await pluginsApi.checkMarketplaceExists(favorite.marketplace_name)
    if (!exists) {
      // 检查是否有保存的市场来源
      if (!favorite.marketplace_source) {
        showCliOutput('市场不存在且无法获取来源信息，请手动安装市场后再试', true)
        return
      }
      // 判断是否为本地路径（包含 :/ 或 \ 或以字母盘开头）
      const isLocalPath = /^[A-Za-z]:[\\\/]/.test(favorite.marketplace_source) ||
                          favorite.marketplace_source.includes('\\') ||
                          !favorite.marketplace_source.includes('/')
      if (isLocalPath) {
        showCliOutput('该插件来自本地市场，无法自动恢复', true)
        return
      }

      showCliOutput('市场不存在，正在安装市场...')
      // 使用保存的市场来源安装市场
      const marketResult = await pluginsApi.addMarketplace(favorite.marketplace_source)
      allPlugins.value = marketResult.plugins
      marketplaceList.value = marketResult.marketplaces
      showCliOutput(marketResult.cli_output || '市场安装成功，正在安装插件...')
    }

    // 安装插件
    const result = await pluginsApi.install(favorite.plugin_id)
    allPlugins.value = result.plugins
    favoriteList.value = await pluginsApi.getFavorites()
    showCliOutput(result.cli_output)
  } catch (error: any) {
    showCliOutput(error?.message || '安装失败', true)
  } finally {
    operatingPlugin.value = null
    operatingType.value = null
  }
}

// 从收藏中移除
async function handleRemoveFavoriteById(favorite: PluginFavoriteItem) {
  operatingPlugin.value = favorite.plugin_id
  operatingType.value = 'unfavorite'
  try {
    const result = await pluginsApi.removeFavorite(favorite.plugin_id)
    allPlugins.value = result.plugins
    favoriteList.value = await pluginsApi.getFavorites()
    showCliOutput('已移除')
  } catch (error: any) {
    showCliOutput(error?.message || '操作失败', true)
  } finally {
    operatingPlugin.value = null
    operatingType.value = null
  }
}

// 添加市场
async function handleAddMarketplace() {
  if (!marketForm.value.url.trim()) {
    showCliOutput('请输入市场 URL', true)
    return
  }
  savingMarket.value = true
  try {
    const result = await pluginsApi.addMarketplace(marketForm.value.url.trim())
    allPlugins.value = result.plugins
    marketplaceList.value = result.marketplaces
    favoriteList.value = await pluginsApi.getFavorites()
    showCliOutput(result.cli_output)
    showAddMarketDialog.value = false
    marketForm.value = { url: '' }
  } catch (error: any) {
    showCliOutput(error?.message || '添加失败', true)
  } finally {
    savingMarket.value = false
  }
}

// 删除市场
async function handleRemoveMarketplace(market: MarketplaceInfo) {
  try {
    await ElMessageBox.confirm(`确定删除市场 "${market.name}"?`, '确认')
    operatingMarket.value = market.name
    operatingMarketType.value = 'remove'
    const result = await pluginsApi.removeMarketplace(market.name)
    allPlugins.value = result.plugins
    marketplaceList.value = result.marketplaces
    favoriteList.value = await pluginsApi.getFavorites()
    showCliOutput(result.cli_output)
  } catch (error: any) {
    if (error !== 'cancel') {
      showCliOutput(error?.message || '删除失败', true)
    }
  } finally {
    operatingMarket.value = null
    operatingMarketType.value = null
  }
}

// 更新市场
async function handleUpdateMarketplace(market: MarketplaceInfo) {
  operatingMarket.value = market.name
  operatingMarketType.value = 'update'
  try {
    const result = await pluginsApi.updateMarketplace(market.name)
    allPlugins.value = result.plugins
    marketplaceList.value = result.marketplaces
    favoriteList.value = await pluginsApi.getFavorites()
    showCliOutput(result.cli_output)
  } catch (error: any) {
    showCliOutput(error?.message || '更新失败', true)
  } finally {
    operatingMarket.value = null
    operatingMarketType.value = null
  }
}

onMounted(() => {
  loadAll()
})
</script>

<style scoped>
.plugins-page {
  height: 100%;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.text-muted {
  color: #909399;
}

.plugin-name-cell {
  display: flex;
  flex-direction: column;
}

.plugin-meta {
  color: var(--el-text-color-secondary);
  font-size: 12px;
}

.action-buttons {
  display: flex;
  flex-wrap: nowrap;
  gap: 8px;
}

.favorites-tip {
  color: var(--el-text-color-secondary);
  font-size: 12px;
  margin-bottom: 16px;
}

.plugin-desc {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.5;
}
</style>

<style>
.plugin-desc-tooltip {
  max-width: 400px;
  word-break: break-word;
}
</style>