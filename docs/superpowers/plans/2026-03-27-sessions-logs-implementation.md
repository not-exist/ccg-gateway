# Sessions & Logs Refactoring Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this completely separated Phase 2 redesign batch.

**Goal:** Implement the newly approved `V4` "Friendly Ethereal Frost" UI redesign strictly across the Sessions and Logs modules, applying the new Ethereal CSS tokens, custom SVG icons, and flattening the tables logic without destroying original interactions.

**Architecture:** Vue 3 Composition API. We will replace large swaths of Element Plus UI wrappers (`el-card`, `el-table`, `el-drawer`, standard `<select>`) with custom Flex containers (`b-card`, `b-segmented`, `.custom-select`) matching the exact pixel specifications of `prototype-sessions-logs-v4.html`.

---

### Task 1: Sessions Module (Project Grid & Detail Flow)

**Files:**
- Modify: `frontend/src/views/sessions/index.vue`

- [ ] **Step 1: Overhaul Project Grid Layer**
Replace the top `el-tabs` with custom HTML tabs (`.top-tabs`).
Convert the `el-row/el-col` project layout into a CSS Grid (`display: grid; grid-template-columns: repeat(2, 1fr)`).
Apply `backdrop-filter: blur(10px)` and soft background styling to exactly match the transparent card vibe.

- [ ] **Step 2: Restructure Session Info Flow**
Inside a project item: Update the `<div class="session-card">` layout to match the exact `V4` 3-Line spec:
- Vertical-center the chat `<svg>`.
- **Line 1:** `ID + Branch`.
- **Line 2:** Strict 1-Line Preview via CSS truncation `white-space: nowrap`.
- **Line 3:** Timestamp and Capacity string at the bottom.

- [ ] **Step 3: Replace Drawer & Chat Bubbles**
Remove `el-drawer` dependencies. Implement a completely custom sliding overlay (`.scrim` + `.drawer`).
Re-style the looping messages inside the Drawer to use high-contrast modern messaging UI: User (Blue Box/#0ea5e9), Assistant (White Box/Border). Setup the inline `copy` `<svg>` action.

- [ ] **Step 4: Verify & Commit**
```bash
git add frontend/src/views/sessions/index.vue
git commit -m "refactor(ui): upgrade sessions module to V4 ethereal grid and custom drawer bubbles"
```


### Task 2: Logs Module (Data Density & Custom Selects)

**Files:**
- Modify: `frontend/src/views/logs/index.vue`

- [ ] **Step 1: Convert Headers and Filters**
Convert the native tabs to `.top-tabs`.
Re-build the HTTP Status top widget ("记录请求日志" toggle) using the new white-blur banner box.

- [ ] **Step 2: Custom Dropdown Injection (Crucial)**
Because native `<select>` breaks the design aesthetic, wrap the `CLI类型` and `路由节点` selection functionality into Vue-managed `.custom-select` structures natively mirroring headless UI components (incorporating the dropdown state toggles smoothly without leaving Vue bounds).

- [ ] **Step 3: Flatten Data Table (`table.flat-table`)**
Remove `<el-table>`. Inject `<table class="flat-table">`.
Attach `white-space: nowrap` strictly to guarantee uniform 1-line height for every log entry.
Change model routing display (`源 → 目标`) to be completely inline text rather than multi-axis layout.
Target only HTTP Status columns (200, 504) with colored `.pill` tags, stripping visual hierarchy entirely off other data fields for scannability.

- [ ] **Step 4: Verify & Commit**
```bash
git add frontend/src/views/logs/index.vue
git commit -m "refactor(ui): upgrade logs module featuring custom select UI and high-density flat table"
```
