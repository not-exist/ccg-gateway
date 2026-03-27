# Global Config Redesign Spec (Friendly & Ethereal Frost)

## 1. Overview
The Global Config (全局设置) view completely sheds the heavy "admin panel" aesthetic of generic Element Plus layouts, matching the unified "Friendly Ethereal Frost" visual language, while meticulously preserving the split-column functionality.

## 2. Layout Architecture
*   **Two-Column Dashboard:**
    *   **Left Column:** "基础配置" (Base Config) and "备份与恢复" (Backup & Restore).
    *   **Right Column:** "CLI配置" (CLI Settings).
*   **Card Styling:** White backgrounds (`#ffffff`), `16px` border-radius, soft floating drop shadows (`box-shadow: 0 4px 12px rgba(0,0,0,0.03)`). Hover states lift the card by `-2px`.

## 3. Component Details

### 3.1 基础配置 (Timeouts)
*   **Inputs:** "流式首字节超时", "流式空闲超时", "非流式超时" with "秒" suffix.
*   **Action:** The **"保存"** (Save) button is strictly aligned to the right side (`justify-content: flex-end`) for universal consistency.

### 3.2 备份与恢复 (Backup & Restore)
*   **Toggle:** Uses a 100% width Segmented Control containing two items: **"本地备份"** and **"WebDAV"**. Each button spans `flex: 1`. 
*   **Local Backup State:** 
    *   Subtext: "将数据库文件导出到本地，或从本地文件恢复".
    *   Buttons: **"导出"** (Green), **"导入"** (White outline).
    *   Buttons strictly aligned to the right (`justify-content: flex-end`).
*   **WebDAV State:**
    *   Inputs: "服务器", "用户名", "密码".
    *   Buttons: **"测试连接"**, **"保存配置"**, **"导出"** (Green), **"导入"** (Amber).
    *   Buttons strictly grouped and aligned to the right (`justify-content: flex-end`).

### 3.3 CLI配置 (CLI Settings)
*   **Toggle:** Uses a 100% width Segmented Control containing three items: **"ClaudeCode"**, **"Codex"**, **"Gemini"**. Each button spans `flex: 1`.
*   **Fields:** 
    *   "配置目录" (Readonly input with purely grey background) + "重置" button.
    *   "默认配置" (`settings.json` or `config.toml` textarea). High-contrast monospaced font with a slight muting (`#f8fafc`).
*   **Dynamic Help Text:**
    *   For ClaudeCode / Gemini: `此处配置会合并到 settings.json（JSON 格式）`.
    *   For Codex: `此处配置会合并到 config.toml（TOML 格式）`.
*   **Action:** Buttons **"格式化"** and **"保存"**, aligned strictly to the right (`justify-content: flex-end`).
