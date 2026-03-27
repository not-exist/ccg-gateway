# Sessions & Logs Redesign (Friendly Ethereal Frost)

## 1. Overview
The "Sessions" (`/sessions`) and "Logs" (`/logs`) modules must be refactored to explicitly eliminate Element Plus heavy framing (such as `el-card`, `el-table`, and `el-tabs`) and adopt the tailored "Ethereal Frost" logic proven in prototype `master-prototype-v3.html` and `prototype-sessions-logs-v4.html`.

## 2. Shared Principles
- **Global Background/Card:** The main view container must use the `linear-gradient(150deg, #f4f7fe 0%, #ffffff 100%)` background with `box-shadow: inset 0 0 0 1px rgba(255,255,255,0.8), 0 10px 40px -10px rgba(0,0,0,0.03)`.
- **Icons:** Eradicate all Unicode Emojis (📁, 💬). Strictly use high-quality Lucide SVG icons matching the `V4` styling.
- **Top Headers:** Replace `el-tabs` with standard HTML flex rows bearing `.top-tabs` and `.tab-item` classes (`padding-bottom: 12px`, `border-bottom: 2px solid`).

## 3. Sessions Module (`frontend/src/views/sessions/index.vue`)
### 3.1 Project List (Grid View)
- Instead of tables/lists, projects are displayed via a unified 2-column flex grid (`grid-template-columns: repeat(2, 1fr)`).
- **Project Card Structure:** Horizontal alignment (`display: flex; align-items: center; gap: 16px;`). Left side bears a `48x48` gradient background box with Lucide's Folder icon.
- Cards must use `backdrop-filter: blur(10px)` on `rgba(255,255,255,0.7)` and feature a `translateY(-2px)` hover effect revealing a right-aligned ghost-delete icon.

### 3.2 Session Detail List (Inner View)
- The header requires a minimal Back arrow button (`<button class="b-button-outline">`) and a refined "搜索会话..." search input.
- **Session Row (3-Line Flow):**
  - **Column 1:** A `40x40` perfectly circular `#f0f9ff` icon box containing the Lucide chat icon, strictly vertically centered.
  - **Column 2:** Complete flex structure parsing the text organically.
    - **Line 1:** Session ID + Branch (blue pill label).
    - **Line 2:** `white-space: nowrap; overflow: hidden; text-overflow: ellipsis;` strict single-line message preview.
    - **Line 3:** Timestamp + File Size rendered safely at the bottom using monospace fonts.

### 3.3 The Chat Drawer UX
- Overhaul the Element `el-drawer`. Use a custom `.drawer` panel animating from the right with `box-shadow: -20px 0 50px rgba(0,0,0,0.05)`.
- Chat Bubbles drastically differ:
  - User messages: Float right, background `#0ea5e9`, text `#ffffff`.
  - Assistant messages: Float left, background `#ffffff`, border `#f1f5f9`, text `#334155`.
- Hovering over a bubble must smoothly reveal an inline `copy` Lucide SVG.

## 4. Logs Module (`frontend/src/views/logs/index.vue`)
### 4.1 Filter Header & "Custom Select" Component
- The standard Windows `<select>` elements look completely repulsive against the ethereal frost theme.
- **Action:** Form elements like "CLI 类型" or "路由节点" must be refactored into robust Vue sub-components (or tailored CSS/JS structures) strictly behaving as CSS-driven `.custom-select` elements simulating Headless UI (floating white bordered boxes with chevron rotational animations and inline checkmarks for active states).

### 4.2 Flat Glass Table
- Exterminate all `el-table` wrappers. Use a pure HTML `table.flat-table`.
- **Absolute Rule:** Every `<td>` MUST be restricted to `white-space: nowrap; overflow: hidden; text-overflow: ellipsis;`. Single line depth is uncompromising.
- **Model Routing Column:** Never break lines. Format the source and target strictly inline across the horizontal plain: `gpt-4-turbo <span style="color:#94a3b8">→</span> deepseek-chat`.
- **Status Pills:** Apply color *only* to the HTTP Status Code (Green `#10b981` for 200, Red `#f43f5e` for 504). All other elements use muted grays and unstyled fonts to allow extreme rapid parsing.
