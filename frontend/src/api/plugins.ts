import { invoke } from '@tauri-apps/api/core'
import type {
  MarketplaceInfo,
  PluginItem,
  PluginFavoriteItem,
  PluginActionResult,
  MarketplaceActionResult
} from '@/types/models'

export interface FavoriteInstallResult {
  cli_output: string
}

export const pluginsApi = {
  // 获取已安装插件列表
  getInstalled: async (): Promise<PluginItem[]> => {
    return await invoke<PluginItem[]>('get_installed_plugins')
  },

  // 获取指定市场的插件列表（按需加载）
  getMarketplacePlugins: async (marketName: string): Promise<PluginItem[]> => {
    return await invoke<PluginItem[]>('get_marketplace_plugins', { marketName })
  },

  // 获取市场列表
  getMarketplaces: async (): Promise<MarketplaceInfo[]> => {
    return await invoke<MarketplaceInfo[]>('get_marketplaces')
  },

  // 插件操作
  pluginAction: async (action: string, pluginId: string): Promise<PluginActionResult> => {
    return await invoke<PluginActionResult>('plugin_action', { action, pluginId })
  },

  // 获取收藏列表
  getFavorites: async (): Promise<PluginFavoriteItem[]> => {
    return await invoke<PluginFavoriteItem[]>('get_plugin_favorites')
  },

  // 添加收藏（返回提示消息，如本地市场警告）
  addFavorite: async (pluginId: string, pluginName: string, marketplaceName: string): Promise<string> => {
    return await invoke<string>('add_plugin_favorite', { pluginId, pluginName, marketplaceName })
  },

  // 移除收藏
  removeFavorite: async (pluginId: string): Promise<void> => {
    await invoke('remove_plugin_favorite', { pluginId })
  },

  // 安装收藏的插件（包含市场检查和安装）
  installFavorite: async (
    pluginId: string,
    marketplaceName: string,
    marketplaceSource?: string
  ): Promise<FavoriteInstallResult> => {
    return await invoke<FavoriteInstallResult>('install_favorite_plugin', {
      pluginId,
      marketplaceName,
      marketplaceSource
    })
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
