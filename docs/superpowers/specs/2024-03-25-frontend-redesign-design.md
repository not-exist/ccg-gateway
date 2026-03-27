# Frontend Redesign Spec: Dashboard (Friendly & Ethereal Frost)

## 1. Overview
The goal is to redesign the Dashboard frontend of the CCG Gateway Tauri application using the "Friendly Ethereal Frost" design system. It eliminates Element Plus default borders and heavy backgrounds, implementing a bespoke, vibrant, and airy aesthetic.

## 2. Design Language & Aesthetics
*   **Base Style:** Friendly Ethereal Frost.
*   **Backgrounds:** A soft blue-grey (`#f4f7fe`) for the main app background, and pure white (`#ffffff`) for content cards with soft box shadows (`box-shadow: 0 4px 12px rgba(0,0,0,0.03)`).
*   **Colors:** Primary/Accent Blue (`#0ea5e9`), Success Green (`#10b981`), Warning Amber (`#f59e0b`), Danger Red (`#f43f5e`). Headings (`#0f172a`), Text (`#475569`, `#94a3b8`).
*   **Shapes:** `16px` border radii for main cards, `8px` for buttons and inputs.
*   **Global Layout:** The application shell MUST be built with `width: 100%` and fluid heights, avoiding hardcoded `max-width: 1400px` to maintain responsivenes on wide monitors. It MUST globally inject `box-sizing: border-box` to prevent padded inputs from overflowing.
*   **Global Sidebar Navigation:** Fully Chinese terms: "总览" (仪表盘, 会话记录, 系统日志), "核心资源" (服务商, MCP 工具, 提示词, 扩展技能), "系统管理" (全局设置).

## 3. Key View Overhaul: Dashboard (仪表盘)
### 3.1 CLI Status Cards (Top Row)
*   **Layout:** 3 parallel white cards (Claude Code, Codex, Gemini).
*   **UI Elements:**
    *   No redundant text ("运行中" / "已停止"). Relies entirely on colored status dots (Green for active, Grey for disabled/off) and text opacity.
    *   **Proxy vs Direct Mode Switch:** Integrated iOS-style segmented control inside the card (`中转模式` / `官方模式`). This is not just a visual tab; it MUST invoke the API (e.g. `setCliMode`) to actively mutate the CLI's proxy routing behavior, accompanied by cursors as `pointer` and loader states.
    *   **Master Toggle:** Friendly iOS-style toggle switches (`.toggle`) in the top right to enable/disable the node.

### 3.2 KPI Overview (Middle Row)
*   **Layout:** 4 distinct high-contrast numerical cards.
*   **Card Styling:** Compressed height design (`padding: 24px 20px`) to prevent them from looking excessively tall.
*   **Exact Labels (Must Match):**
    1.  **请求总数** (Accent Blue text)
    2.  **全局成功率** (Emerald Green text)
    3.  **Token消耗** (Dark Slate text)
    4.  **活跃服务商** (Dark Slate text)
*   **Visuals:** Typography-only. Icons removed. Font size reduced to `32px` (tabular-nums) for balanced scaling.

### 3.3 Data & Charts (Bottom Row)
*   **Layout:** A two-column split (`flex: 2` vs `flex: 1`).
*   **Left Pane - 请求统计与成功率趋势 (Implementation Strategy):**
    *   **Library:** Apache ECharts (`vue-echarts`).
    *   **Design:** A dual-axis smooth line/area chart (`smooth: true`).
    *   **Series 1 (Requests):** Filled area chart using standard Accent Blue (`#0ea5e9`) with a vertical transparency gradient for an "Ethereal" feeling. Mapped to the left Y-axis.
    *   **Series 2 (Success Rate):** Solid smooth line in Emerald Green (`#10b981`). Mapped to the right Y-axis (Percentage).
    *   **Grid:** Hide all vertical grid lines, use only subtle dashed horizontal lines (`#e2e8f0`).
*   **Right Pane - 最近失败记录:** Contains a spaced-out list or table displaying the recent failure logs without heavy borders (`border: 1px dashed #cbd5e1` for empty/loading states).