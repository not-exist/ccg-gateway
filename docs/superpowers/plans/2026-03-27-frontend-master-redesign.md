# Frontend Master Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the newly approved "Friendly Ethereal Frost" UI redesign across the Dashboard, Providers, and Global Config views, transitioning from strict tables to bespoke cards, smooth charts, and Chinese nomenclature.

**Architecture:** Use Vue 3 Composition API and scoped CSS carefully overriding `el-*` components or replacing them entirely with native flex containers styled via utility classes (`b-card`, `b-segmented`, `c-input`). Integrate `vue-echarts` for the gradient dual-axis trend lines.

**Tech Stack:** Vue 3, Element Plus, ECharts (`echarts`, `vue-echarts`).

---

### Task 1: Environment & Main Sidebar Globalization

**Files:**
- Modify: `frontend/package.json`
- Modify: `frontend/src/layouts/MainLayout.vue`

- [ ] **Step 1: Install charting dependencies**
Run the following in the terminal:
```bash
cd frontend && npm install echarts vue-echarts
```

- [ ] **Step 2: Update Sidebar Navigation nomenclature**
Modify `frontend/src/layouts/MainLayout.vue` to use the new categorical Chinese text:
- Identify the `el-menu` blocks.
- Update top-level headers to: `总览` (instead of Overview/Dashboard), `核心资源` (Providers, MCP Tool, etc.), `系统管理` (Config).
- Update sub-items to strictly follow: `仪表盘`, `会话记录`, `系统日志`, `服务商`, `MCP工具`, `全局设置`.

- [ ] **Step 3: Test and Commit**
Start the dev server: `cd frontend && npm run dev`
Verify sidebar names appear correctly translated.
```bash
git add frontend/package.json frontend/package-lock.json frontend/src/layouts/MainLayout.vue
git commit -m "feat(ui): update side menu nomenclature and install echarts"
```


### Task 2: Dashboard Overview & KPI Metrics Refactoring

**Files:**
- Modify: `frontend/src/views/dashboard/index.vue`

- [ ] **Step 1: Overhaul Top CLI Cards**
Remove the old node tables. Create three `b-card` flex containers for `Claude Code`, `Codex`, and `Gemini`.
Each card should contain:
- Title + green/grey dot indicator.
- Master toggle (`el-switch` or custom toggle).
- Integrated segmented control (using standard `flex` row, text `中转模式` / `官方模式`).

- [ ] **Step 2: Overhaul Middle KPI Cards**
Replace old counters with 4 highly compressed static-styled cards:
```html
<div class="kpi-panel" style="display: flex; gap: 24px; margin-bottom: 24px;">
  <!-- Render these exact 4 blocks with padding: 24px 20px -->
  <div class="b-card kpi-card">
    <div class="kpi-title">请求总数</div>
    <div class="kpi-value text-blue">...</div>
  </div>
  <div class="b-card kpi-card">
    <div class="kpi-title">全局成功率</div>
    <div class="kpi-value text-green">...</div>
  </div>
  <div class="b-card kpi-card">
    <div class="kpi-title">Token消耗</div>
    <div class="kpi-value">...</div>
  </div>
  <div class="b-card kpi-card">
    <div class="kpi-title">活跃服务商</div>
    <div class="kpi-value">...</div>
  </div>
</div>
```

- [ ] **Step 3: Verify & Commit**
```bash
git add frontend/src/views/dashboard/index.vue
git commit -m "refactor(dashboard): implement frosty ui top nodes and compressed KPI cards"
```


### Task 3: Dashboard ECharts Integration

**Files:**
- Modify: `frontend/src/views/dashboard/index.vue`
- Modify: `frontend/src/main.ts` (if needed to register `v-chart` globally, or just import locally)

- [ ] **Step 1: Import ECharts Locally**
In `index.vue`:
```typescript
import { use } from 'echarts/core'
import { LineChart, BarChart } from 'echarts/charts'
import { TooltipComponent, GridComponent, DatasetComponent, TransformComponent } from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import VChart from 'vue-echarts'

use([LineChart, BarChart, TooltipComponent, GridComponent, DatasetComponent, TransformComponent, CanvasRenderer])
```

- [ ] **Step 2: Assemble Dual-Axis Charts**
Create the bottom two-column layout (`flex: 2` vs `flex: 1`).
Insert `<v-chart class="chart" :option="chartOption" autoresize />` in the left pane.
Configure `chartOption` with:
- `yAxis: [{ type: 'value' }, { type: 'value', max: 100 }]`
- Hide vertical grid lines completely (grid line style: dashed horizontal `#e2e8f0`).
- Series 1 (Requests): `areaStyle: { color: new echarts.graphic.LinearGradient(...) }`, `smooth: true`, color `#0ea5e9`.
- Series 2 (Success Rate): `yAxisIndex: 1`, `smooth: true`, color `#10b981`.

- [ ] **Step 3: Setup Right Pane (Recent Failures)**
Create a simple padded list on the right pane representing `最近失败记录`.

- [ ] **Step 4: Verify & Commit**
```bash
git add frontend/src/views/dashboard/index.vue
git commit -m "feat(dashboard): integrate echarts dual-axis trend and failures schema"
```


### Task 4: Providers Module Structure and List Re-skin

**Files:**
- Modify: `frontend/src/views/providers/index.vue`

- [ ] **Step 1: Simplify Tabs & "Add Provider" Header**
Replace `el-tabs` with blue-underline styled headers.
Update the proxy/direct radio/switch to entirely custom segmented pills: `中转模式` and `官方模式`. Explicitly remove any "全部" options.

- [ ] **Step 2: Inject Drag Handles & Base URL into List Items**
Update the loop rendered items inside `<draggable>`:
- Add a drag handle (3 distinct dots vertically) on the far left.
- Add `<div class="provider-url">{{ element.base_url }}</div>` below the provider name.
- Change labels "失败" to `失败次数` and `失败阈值` using two distinct stacked text columns on the right.

- [ ] **Step 3: Permanent Blue Edit Button**
Modify the row so that even if `element.enabled === false` (making the row slightly opaque/tagged "已禁用"), the `编辑` button retains full opacity and `#0ea5e9` text over a `#f0f9ff` background.

- [ ] **Step 4: Verify & Commit**
```bash
git add frontend/src/views/providers/index.vue
git commit -m "refactor(providers): overhaul list visuals with explicit handles and aligned stats"
```


### Task 5: Providers Advanced Modal Rework

**Files:**
- Modify: `frontend/src/views/providers/index.vue`

- [ ] **Step 1: Form UI Replacement**
Rewrite the `el-dialog` block for editing providers. Transform standard `el-form-item` wrappers into bespoke `c-label` and `c-input` HTML structural flows mimicking the UI layout:
- Horizontal split for `Name` and `Base URL`.
- Sub-block grey container (`background: #f8fafc`) housing `失败鉴权阈值`, `拉黑时长`, and `自定义 UA` side-by-side.
- Remove legacy borders and ensure a seamless white layout.

- [ ] **Step 2: Model Mapping Array Enhancements**
Convert the Model Mapping input array styling:
- Replace standard cards with tight flex rows of (`source` input -> `→` text arrow -> `target` input -> `×` button).
- Apply similar row styles to the Model Blacklist array.

- [ ] **Step 3: Functional Form Connectivity**
Ensure `v-model` binding correctly touches `form.name`, `form.base_url`, `form.api_key`, `form.failure_threshold`, `form.blacklist_minutes`, etc. Keep existing `@click="handleSave"` hooks intact.

- [ ] **Step 4: Verify & Commit**
```bash
git add frontend/src/views/providers/index.vue
git commit -m "refactor(providers): construct high-end bespoke provider editing modal"
```


### Task 6: Global Config Structural Adjustments

**Files:**
- Modify: `frontend/src/views/config/index.vue`
- Modify: `frontend/src/views/config/components/CliSettingsForm.vue`

- [ ] **Step 1: Base Configuration Card Fixes**
In `frontend/src/views/config/index.vue`:
Translate headers to match spec ("流式首字节超时", "流式空闲超时", "非流式超时"). Make the inputs look like standard `c-input` inline boxes. Force the "保存" primary action button to strictly `justify-content: flex-end`.

- [ ] **Step 2: Backup Segment Refactoring**
Update the "备份与恢复" section. Replace traditional tabs returning to a single full-width segmented control ("本地备份", "WebDAV").
Check specific tab states: In WebDAV, organize inputs neatly and ensure all final buttons ("测试连接", "保存配置", "导出", "导入") are pushed `flex-end` to the right inside the card.

- [ ] **Step 3: CLI Card Restructuring**
Open `CliSettingsForm.vue`. 
Transform the layout to match the right-side overarching card concept.
- Switch the CLI target selector to the bespoke `b-segmented` UI ("ClaudeCode", "Codex", "Gemini") spanning 100% width.
- Ensure the explanatory texts dynamically adapt ("合并到 settings.json（JSON 格式）" vs "合并到 config.toml（TOML 格式）").
- Place "格式化" and "保存" buttons strictly to the right.

- [ ] **Step 4: Verify & Commit**
```bash
git add frontend/src/views/config/index.vue frontend/src/views/config/components/CliSettingsForm.vue
git commit -m "refactor(config): update global settings dual-column layout and finalize segmented controls"
```

## Post-Implementation Checklist
- [x] All 3 specifications covered.
- [x] Exact terminology (请求总数, Token消耗) applied.
- [x] ECharts technical details explicitly provided.
- [x] Clean bite-sized subagent logic mapping intact.
