# Plugin Management Cache Refactoring Design

**Goal:** Refactor the Plugin management system to eliminate the monolithic backend cache file (`plugins_cache.json`) and transition to an on-demand, just-in-time loading architecture, bringing it in line with the current Skill management implementation.

## Architecture

The current system relies on an eager loading approach where all plugins from all installed marketplaces are fetched, evaluated against local installation state, cached to a JSON file, and then returned in bulk to the frontend. The frontend then filters this large payload based on the active tab.

The new architecture will move to a lazy, on-demand loading model:

1.  **Backend Simplification**: The `PluginsCache` struct and all logic related to writing/reading `plugins_cache.json` in `src-tauri/src/services/plugin.rs` will be removed.
2.  **Granular API Endpoints**: The backend will expose specific endpoints tailored to the frontend's views:
    *   `get_installed_plugins`: Returns only plugins that are currently installed (by running `claude plugin list` and enriching with marketplace data).
    *   `get_marketplaces`: Returns the list of configured marketplaces.
    *   `get_marketplace_plugins(marketplace_name)`: Returns the plugins available in a specific marketplace, annotating which ones are installed.
    *   `get_plugin_favorites`: Returns the user's favorited plugins.
3.  **Frontend Alignment**: The frontend `pluginsApi` and `index.vue` will be updated to call these specific endpoints only when the user navigates to the relevant tab or view.

## Components

### Backend (`src-tauri/src/services/plugin.rs` & `commands.rs`)

*   **Remove Cache Logic**: Delete `CACHE_LOCK`, `PluginsCache`, `get_cache_path`, `write_cache`, `read_cache`, `generate_cache`, `update_installed_status`, `update_favorite_status`.
*   **New API: `get_installed_plugins`**:
    *   Runs `get_installed_plugins_sync()`.
    *   Iterates over known marketplaces to find descriptions/details for the installed plugins.
    *   Returns a `Vec<PluginItem>` containing only installed plugins.
*   **New API: `get_marketplace_plugins(market_name)`**:
    *   Runs `get_installed_plugins_sync()` to get current state.
    *   Reads the `.claude-plugin/plugins.json` for the specified `market_name`.
    *   Maps the marketplace plugins to `PluginItem`, setting the `is_installed` flag based on the sync data.
*   **Update `plugin_action`**: After performing an action (install/uninstall), instead of updating a global cache, it can either return the updated status for the specific plugin, or the frontend can re-fetch the current view's data. To minimize frontend changes, it will return success, and the frontend will refresh its active list.
*   **Favorites Logic**: The favorites DB table remains. `get_plugin_favorites` will map the DB rows to `PluginFavoriteItem` by doing a quick check against installed state (via `get_installed_plugins_sync()`) and marketplace data.

### Frontend (`frontend/src/api/plugins.ts` & `frontend/src/views/plugins/index.vue`)

*   **Update `pluginsApi`**:
    *   Replace `getAll` with `getInstalled`.
    *   Add `getMarketplacePlugins(marketName)`.
    *   Adjust return types of action endpoints if necessary (e.g., if they no longer return the monolithic `allPlugins` array).
*   **Update `index.vue`**:
    *   Remove `allPlugins` state.
    *   Add specific state variables: `installedPlugins` and `marketPlugins` (instead of computed properties based on `allPlugins`).
    *   **Tab "插件" (Installed)**: Call `pluginsApi.getInstalled()` when mounted or when switching to this tab.
    *   **Tab "市场" (Marketplaces)**: Call `pluginsApi.getMarketplaces()` to show the list of markets.
    *   **Marketplace Detail View**: When clicking a market, call `pluginsApi.getMarketplacePlugins(market.name)`.
    *   **Actions (Install/Uninstall/Toggle)**: After an action completes, refresh the specific list that is currently visible (e.g., re-fetch installed plugins if on the Installed tab, or re-fetch market plugins if viewing a market).

## Data Flow

**Viewing Installed Plugins:**
Frontend (`getInstalled`) -> Backend (`get_installed_plugins`) -> Backend runs `claude plugin list` + reads market JSONs -> Returns filtered list -> Frontend renders.

**Viewing a Marketplace:**
Frontend (`getMarketplacePlugins(name)`) -> Backend (`get_marketplace_plugins(name)`) -> Backend runs `claude plugin list` + reads specific market JSON -> Returns list for that market -> Frontend renders.

## Error Handling

*   Backend functions will propagate `Result<T>` back to Tauri commands, which convert to string errors for the frontend as currently implemented.
*   If reading a specific marketplace's `plugins.json` fails, `get_marketplace_plugins` should return a clear error indicating the marketplace might be corrupted or needs syncing.

## Testing

*   Verify that navigating between tabs correctly loads only the necessary data.
*   Install a plugin from a marketplace: verify the button changes state, and that the plugin appears in the "Installed" tab.
*   Uninstall a plugin from the Installed tab: verify it disappears from the list.
*   Verify favorites can be added, removed, and installed correctly.