import { invoke } from '@tauri-apps/api/core'
import type {
  MarketplaceInfo,
  PluginItem,
  PluginFavoriteItem,
  PluginActionResult,
  MarketplaceActionResult
} from '@/types/models'

export const pluginsApi = {
  // 获取插件列表
  getAll: async (): Promise<PluginItem[]> => {
    return await invoke<PluginItem[]>('get_all_plugins')
  },

  // 获取市场列表
  getMarketplaces: async (): Promise<MarketplaceInfo[]> => {
    return await invoke<MarketplaceInfo[]>('get_marketplaces')
  },

  // 刷新插件列表
  refresh: async (): Promise<PluginItem[]> => {
    return await invoke<PluginItem[]>('refresh_plugins')
  },

  // 插件操作
  pluginAction: async (action: string, pluginId: string): Promise<PluginActionResult> => {
    return await invoke<PluginActionResult>('plugin_action', { action, pluginId })
  },

  // 获取收藏列表
  getFavorites: async (): Promise<PluginFavoriteItem[]> => {
    return await invoke<PluginFavoriteItem[]>('get_plugin_favorites')
  },

  // 添加收藏（返回更新后的插件列表）
  addFavorite: async (pluginId: string, pluginName: string, marketplaceName: string): Promise<PluginActionResult> => {
    return await invoke<PluginActionResult>('add_plugin_favorite', { pluginId, pluginName, marketplaceName })
  },

  // 移除收藏（返回更新后的插件列表）
  removeFavorite: async (pluginId: string): Promise<PluginActionResult> => {
    return await invoke<PluginActionResult>('remove_plugin_favorite', { pluginId })
  },

  // 检查市场是否存在
  checkMarketplaceExists: async (marketplaceName: string): Promise<boolean> => {
    return await invoke<boolean>('check_marketplace_exists', { marketplaceName })
  },

  // 市场操作
  marketplaceAction: async (action: string, param: string): Promise<MarketplaceActionResult> => {
    return await invoke<MarketplaceActionResult>('marketplace_action', { action, param })
  },

  // 便捷方法
  install: (pluginId: string) => pluginsApi.pluginAction('install', pluginId),
  uninstall: (pluginId: string) => pluginsApi.pluginAction('uninstall', pluginId),
  enable: (pluginId: string) => pluginsApi.pluginAction('enable', pluginId),
  disable: (pluginId: string) => pluginsApi.pluginAction('disable', pluginId),
  update: (pluginId: string) => pluginsApi.pluginAction('update', pluginId),

  addMarketplace: (url: string) => pluginsApi.marketplaceAction('add', url),
  removeMarketplace: (name: string) => pluginsApi.marketplaceAction('remove', name),
  updateMarketplace: (name: string) => pluginsApi.marketplaceAction('update', name)
}