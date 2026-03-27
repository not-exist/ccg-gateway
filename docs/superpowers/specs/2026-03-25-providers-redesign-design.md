# Providers Module Redesign Spec (Friendly & Ethereal Frost)

## 1. Overview
The `Providers` management view is overhauled to align with the "Friendly Ethereal Frost" design language. It replaces Element Plus lists and heavy dialogs with a customized, interactive aesthetic separating CLI environments and mode toggles.

## 2. Page Structure
*   **Left Sidebar/Global Tabs:** Claude Code, Codex, Gemini (Blue bottom underline active state).
*   **Header Controls:** 
    *   Segmented Control explicitly offering ONLY two options: `中转模式` | `官方模式`. (No "全部" filter).
    *   Primary action button: `+ 添加服务商` (Accent Blue).

## 3. Provider List (Card View)
*   **Container:** Unified floating card container (`background: #ffffff`, `radius-lg`).
*   **Row Items Structure:**
    *   **Drag Handle:** The very left of the row features a `⋮⋮` drag handle (dots with `opacity: 0.3`, hover `0.8`) indicating the list supports drag-and-drop ordering.
    *   **Provider Info:** 
        *   Provider Name (`#0f172a`, `16px`, `font-weight: 600`).
        *   Base URL displayed directly below the name in a smaller monospace grey text (`13px`, `#64748b`).
    *   **Tags:** Soft pill-shaped tags next to the name: e.g., "5个模型映射" (green), "已拉黑 (12分后解除)" (red), "已禁用" (grey).
    *   **Status Metrics (Exact Labels):** Two distinct right-aligned metrics: 
        *   `失败次数` (Current failure count: e.g., 0 or 5). Turns red if equal to threshold.
        *   `失败阈值` (Threshold max: e.g., 10 or 5).
    *   **Actions:** 
        *   iOS-style toggle (Enabled/Disabled).
        *   `编辑` (Edit) Button: Has a permanent soft blue background (`#f0f9ff`) and blue text (`#0ea5e9`). It is ALWAYS fully visible/clickable, even if the provider toggle is turned off (unaffected by row opacity).

## 4. Add/Edit Provider Modal
*   **Backdrop:** Blurred overlay (`backdrop-filter: blur(4px)`) over the application.
*   **Container:** White floating dialog (`border-radius: 20px`, `width: 720px`).
*   **Form Design:**
    *   **Base Info:** "服务商名称", "Base URL", "API Key / Token" (fields span flexibly).
    *   **Advanced Group:** A distinct grey bounded box (`#f8fafc`, `border-radius: 12px`, padding inside) holding three specific inputs aligned horizontally:
        *   `失败鉴权阈值 (次)` - (default: 3).
        *   `拉黑时长 (分钟)` - (default: 10).
        *   `自定义 UA (选填)`.
    *   **Model Forwarding (模型转发配置 - 映射):**
        *   Header with title and "+ 添加映射" outline button.
        *   Rows containing: `source_model` input, a visual `→` arrow, `target_model` input, and a circular red `×` button for deletion.
    *   **Model Blacklist (模型黑名单):**
        *   Header with title and "+ 加黑名单" outline button.
        *   Rows containing regex/pattern input and `×` delete button.
    *   **Actions:** "取消修改", "保存配置" (Bottom right, flex-end).