import { invoke } from '@tauri-apps/api/core'
import type {
  InstalledPlugin,
  MarketplaceInfo,
  PluginItem,
  PluginFavorite,
  PluginFavoriteCreate
} from '@/types/models'

export const pluginsApi = {
  // ==================== 插件管理 ====================
  getInstalled: async (): Promise<InstalledPlugin[]> => {
    return await invoke<InstalledPlugin[]>('get_installed_plugins')
  },

  getAll: async (): Promise<PluginItem[]> => {
    return await invoke<PluginItem[]>('get_all_plugins')
  },

  install: async (pluginId: string): Promise<void> => {
    await invoke('install_plugin', { pluginId })
  },

  uninstall: async (pluginId: string): Promise<void> => {
    await invoke('uninstall_plugin', { pluginId })
  },

  enable: async (pluginId: string): Promise<void> => {
    await invoke('enable_plugin', { pluginId })
  },

  disable: async (pluginId: string): Promise<void> => {
    await invoke('disable_plugin', { pluginId })
  },

  update: async (pluginId: string): Promise<void> => {
    await invoke('update_plugin', { pluginId })
  },

  // ==================== 市场管理 ====================
  getMarketplaces: async (): Promise<MarketplaceInfo[]> => {
    return await invoke<MarketplaceInfo[]>('get_installed_marketplaces')
  },

  addMarketplace: async (url: string): Promise<MarketplaceInfo> => {
    return await invoke<MarketplaceInfo>('add_marketplace', { url })
  },

  removeMarketplace: async (name: string): Promise<void> => {
    await invoke('remove_marketplace', { name })
  },

  updateMarketplace: async (name: string): Promise<void> => {
    await invoke('update_marketplace', { name })
  },

  checkMarketplaceExists: async (name: string): Promise<boolean> => {
    return await invoke<boolean>('check_marketplace_exists', { name })
  },

  // ==================== 收藏管理 ====================
  getFavorites: async (): Promise<PluginFavorite[]> => {
    return await invoke<PluginFavorite[]>('get_plugin_favorites')
  },

  addFavorite: async (input: PluginFavoriteCreate): Promise<void> => {
    await invoke('add_plugin_favorite', { input })
  },

  removeFavorite: async (pluginId: string): Promise<void> => {
    await invoke('remove_plugin_favorite', { pluginId })
  },

  installFromFavorite: async (pluginId: string): Promise<void> => {
    await invoke('install_from_favorite', { pluginId })
  }
}