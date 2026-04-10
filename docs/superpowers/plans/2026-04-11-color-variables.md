# 颜色变量系统实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将前端所有硬编码颜色值替换为 CSS 变量，提升可维护性

**Architecture:** 在 App.vue 的 `:root` 定义 55 个语义化颜色变量，然后在各 Vue 组件中将硬编码色值替换为 `var(--color-*)` 引用

**Tech Stack:** Vue 3, CSS Custom Properties

---

## 文件结构

### 修改文件

| 文件 | 职责 |
|---|---|
| `frontend/src/App.vue` | 新增颜色变量定义 |
| `frontend/src/views/config/components/CliSettingsForm.vue` | 替换颜色 |
| `frontend/src/views/config/index.vue` | 替换颜色 |
| `frontend/src/views/dashboard/index.vue` | 替换颜色 |
| `frontend/src/views/logs/index.vue` | 替换颜色 |
| `frontend/src/views/mcp/index.vue` | 替换颜色 |
| `frontend/src/views/plugins/index.vue` | 替换颜色 |
| `frontend/src/views/prompts/index.vue` | 替换颜色 |
| `frontend/src/views/providers/index.vue` | 替换颜色 |
| `frontend/src/views/sessions/index.vue` | 替换颜色 |
| `frontend/src/views/skills/index.vue` | 替换颜色 |
| `frontend/src/components/AppModal.vue` | 替换颜色 |
| `frontend/src/layouts/MainLayout.vue` | 替换颜色 |

---

## Task 1: 在 App.vue 定义颜色变量

**Files:**
- Modify: `frontend/src/App.vue`

- [ ] **Step 1: 在 `:root` 中添加颜色变量定义**

在现有字体变量后添加完整的颜色变量定义：

```css
:root {
  /* ========== 字体规范 ========== */
  /* 字体权重 */
  --fw-400: 400;
  --fw-500: 500;
  --fw-600: 600;
  --fw-700: 700;

  /* 字体大小 */
  --fs-12: 12px;
  --fs-14: 14px;
  --fs-16: 16px;
  --fs-20: 20px;
  --fs-24: 24px;
  --fs-32: 32px;

  /* ========== 颜色规范 ========== */
  /* 主色系 */
  --color-primary: #0ea5e9;
  --color-primary-hover: #0284c7;
  --color-primary-dark: #0369a1;
  --color-primary-light: #f0f9ff;
  --color-primary-lighter: #e0f2fe;
  --color-primary-border: #bae6fd;
  --color-primary-muted: #7dd3fc;
  --color-primary-5: rgba(14, 165, 233, 0.05);
  --color-primary-10: rgba(14, 165, 233, 0.1);
  --color-primary-20: rgba(14, 165, 233, 0.2);
  --color-primary-30: rgba(14, 165, 233, 0.3);

  /* 文字色系 */
  --color-text: #0f172a;
  --color-text-secondary: #475569;
  --color-text-muted: #64748b;
  --color-text-weak: #94a3b8;
  --color-text-dark: #334155;

  /* 背景色系 */
  --color-bg: #ffffff;
  --color-bg-page: #f8fafc;
  --color-bg-subtle: #f1f5f9;
  --color-bg-80: rgba(255, 255, 255, 0.8);
  --color-bg-90: rgba(255, 255, 255, 0.9);
  --color-bg-95: rgba(255, 255, 255, 0.95);

  /* 边框色系 */
  --color-border: #e2e8f0;
  --color-border-hover: #cbd5e1;
  --color-border-light: rgba(226, 232, 240, 0.6);
  --color-border-medium: rgba(226, 232, 240, 0.8);
  --color-scrollbar: #cbd5e1;
  --color-scrollbar-hover: #94a3b8;

  /* 状态色系 */
  --color-success: #10b981;
  --color-success-hover: #059669;
  --color-success-light: #ecfdf5;
  --color-success-10: rgba(16, 185, 129, 0.1);
  --color-success-30: rgba(16, 185, 129, 0.3);
  --color-success-40: rgba(16, 185, 129, 0.4);
  --color-danger: #ef4444;
  --color-danger-hover: #dc2626;
  --color-danger-light: #fee2e2;
  --color-danger-muted: #fca5a5;
  --color-error: #f43f5e;
  --color-error-light: #fff1f2;
  --color-error-2: rgba(244, 63, 94, 0.02);
  --color-error-10: rgba(244, 63, 94, 0.1);
  --color-warning: #f59e0b;
  --color-warning-10: rgba(245, 158, 11, 0.1);

  /* 阴影色系 */
  --color-shadow: rgba(0, 0, 0, 0.03);
  --color-shadow-hover: rgba(0, 0, 0, 0.05);
  --color-shadow-md: rgba(0, 0, 0, 0.08);
  --color-shadow-lg: rgba(0, 0, 0, 0.1);
  --color-shadow-xl: rgba(0, 0, 0, 0.2);
  --color-scrim: rgba(15, 23, 42, 0.25);
  --color-scrim-dark: rgba(15, 23, 42, 0.4);
  --color-overlay: rgba(148, 163, 184, 0.05);
  --color-overlay-8: rgba(148, 163, 184, 0.08);

  /* 特殊色系 */
  --color-violet: #8b5cf6;
  --color-violet-light: #f5f3ff;
}
```

- [ ] **Step 2: 更新现有颜色类使用变量**

将现有颜色类改为使用变量：

```css
/* ========== 颜色类 ========== */
.text-muted { color: var(--color-text-weak); }
.text-secondary { color: var(--color-text-muted); }
.text-primary { color: var(--color-text); }
.text-info { color: var(--color-primary); }
```

- [ ] **Step 3: 更新滚动条样式使用变量**

```css
/* ========== Global Ethereal Scrollbar ========== */
::-webkit-scrollbar {
  width: 14px;
  height: 14px;
}

::-webkit-scrollbar-thumb {
  background-color: var(--color-scrollbar);
  border-radius: 10px;
  border: 4px solid transparent;
  background-clip: padding-box;
}

::-webkit-scrollbar-thumb:hover {
  background-color: var(--color-scrollbar-hover);
}

::-webkit-scrollbar-track {
  background-color: transparent;
}
```

- [ ] **Step 4: 提交**

```bash
git add frontend/src/App.vue
git commit -m "feat: 添加全局颜色变量定义

定义 55 个语义化颜色变量，覆盖主色、文字、背景、边框、状态、阴影、特殊色系"
```

---

## Task 2: 替换 MainLayout.vue 中的颜色

**Files:**
- Modify: `frontend/src/layouts/MainLayout.vue`

- [ ] **Step 1: 替换所有硬编码颜色为变量**

按以下映射替换：
- `#0f172a` → `var(--color-text)`
- `#475569` → `var(--color-text-secondary)`
- `#64748b` → `var(--color-text-muted)`
- `#94a3b8` → `var(--color-text-weak)`
- `#0ea5e9` → `var(--color-primary)`
- `#0284c7` → `var(--color-primary-hover)`
- `#f8fafc` → `var(--color-bg-page)`
- `#f1f5f9` → `var(--color-bg-subtle)`
- `#e2e8f0` → `var(--color-border)`
- `#ffffff` / `white` → `var(--color-bg)`
- `#f4f7fe` → `var(--color-bg-subtle)`
- `rgba(15, 23, 42, 0.25)` → `var(--color-scrim)`
- `rgba(255, 255, 255, 0.95)` → `var(--color-bg-95)`
- `rgba(255, 255, 255, 0.8)` → `var(--color-bg-80)`
- `rgba(0, 0, 0, 0.1)` → `var(--color-shadow-lg)`
- `rgba(226, 232, 240, 0.8)` → `var(--color-border-medium)`
- `rgba(148, 163, 184, 0.05)` → `var(--color-overlay)`
- `rgba(0, 0, 0, 0.03)` → `var(--color-shadow)`
- `rgba(0, 0, 0, 0.05)` → `var(--color-shadow-hover)`

- [ ] **Step 2: 提交**

```bash
git add frontend/src/layouts/MainLayout.vue
git commit -m "refactor: MainLayout 使用颜色变量"
```

---

## Task 3: 替换 AppModal.vue 中的颜色

**Files:**
- Modify: `frontend/src/components/AppModal.vue`

- [ ] **Step 1: 替换所有硬编码颜色为变量**

按映射替换：
- `#0f172a` → `var(--color-text)`
- `#64748b` → `var(--color-text-muted)`
- `#94a3b8` → `var(--color-text-weak)`
- `#0ea5e9` → `var(--color-primary)`
- `#0284c7` → `var(--color-primary-hover)`
- `#ffffff` → `var(--color-bg)`
- `#f8fafc` → `var(--color-bg-page)`
- `#f1f5f9` → `var(--color-bg-subtle)`
- `#e2e8f0` → `var(--color-border)`
- `#cbd5e1` → `var(--color-border-hover)`
- `rgba(15, 23, 42, 0.4)` → `var(--color-scrim-dark)`
- `rgba(0, 0, 0, 0.2)` → `var(--color-shadow-xl)`

- [ ] **Step 2: 提交**

```bash
git add frontend/src/components/AppModal.vue
git commit -m "refactor: AppModal 使用颜色变量"
```

---

## Task 4: 替换 config/index.vue 中的颜色

**Files:**
- Modify: `frontend/src/views/config/index.vue`

- [ ] **Step 1: 替换所有硬编码颜色为变量**

按映射替换（包括 template 内联样式和 style 块）

- [ ] **Step 2: 提交**

```bash
git add frontend/src/views/config/index.vue
git commit -m "refactor: config 页面使用颜色变量"
```

---

## Task 5: 替换 CliSettingsForm.vue 中的颜色

**Files:**
- Modify: `frontend/src/views/config/components/CliSettingsForm.vue`

- [ ] **Step 1: 替换所有硬编码颜色为变量**

- [ ] **Step 2: 提交**

```bash
git add frontend/src/views/config/components/CliSettingsForm.vue
git commit -m "refactor: CliSettingsForm 使用颜色变量"
```

---

## Task 6: 替换 dashboard/index.vue 中的颜色

**Files:**
- Modify: `frontend/src/views/dashboard/index.vue`

- [ ] **Step 1: 替换所有硬编码颜色为变量**

注意：ECharts 图表配置中的颜色也需要替换

- [ ] **Step 2: 提交**

```bash
git add frontend/src/views/dashboard/index.vue
git commit -m "refactor: dashboard 页面使用颜色变量"
```

---

## Task 7: 替换 logs/index.vue 中的颜色

**Files:**
- Modify: `frontend/src/views/logs/index.vue`

- [ ] **Step 1: 替换所有硬编码颜色为变量**

- [ ] **Step 2: 提交**

```bash
git add frontend/src/views/logs/index.vue
git commit -m "refactor: logs 页面使用颜色变量"
```

---

## Task 8: 替换 mcp/index.vue 中的颜色

**Files:**
- Modify: `frontend/src/views/mcp/index.vue`

- [ ] **Step 1: 替换所有硬编码颜色为变量**

- [ ] **Step 2: 提交**

```bash
git add frontend/src/views/mcp/index.vue
git commit -m "refactor: mcp 页面使用颜色变量"
```

---

## Task 9: 替换 plugins/index.vue 中的颜色

**Files:**
- Modify: `frontend/src/views/plugins/index.vue`

- [ ] **Step 1: 替换所有硬编码颜色为变量**

- [ ] **Step 2: 提交**

```bash
git add frontend/src/views/plugins/index.vue
git commit -m "refactor: plugins 页面使用颜色变量"
```

---

## Task 10: 替换 prompts/index.vue 中的颜色

**Files:**
- Modify: `frontend/src/views/prompts/index.vue`

- [ ] **Step 1: 替换所有硬编码颜色为变量**

- [ ] **Step 2: 提交**

```bash
git add frontend/src/views/prompts/index.vue
git commit -m "refactor: prompts 页面使用颜色变量"
```

---

## Task 11: 替换 providers/index.vue 中的颜色

**Files:**
- Modify: `frontend/src/views/providers/index.vue`

- [ ] **Step 1: 替换所有硬编码颜色为变量**

注意：此文件颜色最多，包括 template 内联样式和 style 块

- [ ] **Step 2: 提交**

```bash
git add frontend/src/views/providers/index.vue
git commit -m "refactor: providers 页面使用颜色变量"
```

---

## Task 12: 替换 sessions/index.vue 中的颜色

**Files:**
- Modify: `frontend/src/views/sessions/index.vue`

- [ ] **Step 1: 替换所有硬编码颜色为变量**

- [ ] **Step 2: 提交**

```bash
git add frontend/src/views/sessions/index.vue
git commit -m "refactor: sessions 页面使用颜色变量"
```

---

## Task 13: 替换 skills/index.vue 中的颜色

**Files:**
- Modify: `frontend/src/views/skills/index.vue`

- [ ] **Step 1: 替换所有硬编码颜色为变量**

注意：包含 violet 特殊色系

- [ ] **Step 2: 提交**

```bash
git add frontend/src/views/skills/index.vue
git commit -m "refactor: skills 页面使用颜色变量"
```

---

## Task 14: 验证并清理

**Files:**
- 所有修改的文件

- [ ] **Step 1: 检查是否有遗漏的硬编码颜色**

运行检查命令，搜索残留的硬编码颜色：

```bash
cd frontend/src && grep -rn "#[0-9a-fA-F]\{6\}" --include="*.vue" | grep -v "var(--" || echo "无遗漏"
```

- [ ] **Step 2: 检查 rgba 遗漏**

```bash
cd frontend/src && grep -rn "rgba(" --include="*.vue" | grep -v "var(--" || echo "无遗漏"
```

- [ ] **Step 3: 视觉验证**

启动开发服务器，检查各页面视觉表现是否一致

- [ ] **Step 4: 最终提交**

```bash
git add -A
git commit -m "refactor: 完成颜色变量系统迁移

所有硬编码颜色值已替换为 CSS 变量引用"
```

---

## 颜色映射速查表

| 原值 | 变量 |
|---|---|
| `#0ea5e9` | `var(--color-primary)` |
| `#0284c7` | `var(--color-primary-hover)` |
| `#0369a1` | `var(--color-primary-dark)` |
| `#f0f9ff` | `var(--color-primary-light)` |
| `#e0f2fe` | `var(--color-primary-lighter)` |
| `#bae6fd` | `var(--color-primary-border)` |
| `#7dd3fc` | `var(--color-primary-muted)` |
| `#0f172a` | `var(--color-text)` |
| `#475569` | `var(--color-text-secondary)` |
| `#64748b` | `var(--color-text-muted)` |
| `#94a3b8` | `var(--color-text-weak)` |
| `#334155` | `var(--color-text-dark)` |
| `#ffffff` / `white` | `var(--color-bg)` |
| `#f8fafc` | `var(--color-bg-page)` |
| `#f1f5f9` | `var(--color-bg-subtle)` |
| `#f4f7fe` | `var(--color-bg-subtle)` |
| `#e2e8f0` | `var(--color-border)` |
| `#cbd5e1` | `var(--color-border-hover)` |
| `#10b981` | `var(--color-success)` |
| `#059669` | `var(--color-success-hover)` |
| `#ecfdf5` | `var(--color-success-light)` |
| `#ef4444` | `var(--color-danger)` |
| `#dc2626` | `var(--color-danger-hover)` |
| `#fee2e2` | `var(--color-danger-light)` |
| `#fca5a5` | `var(--color-danger-muted)` |
| `#f43f5e` | `var(--color-error)` |
| `#fff1f2` | `var(--color-error-light)` |
| `#f59e0b` | `var(--color-warning)` |
| `#8b5cf6` | `var(--color-violet)` |
| `#f5f3ff` | `var(--color-violet-light)` |
| `rgba(14, 165, 233, 0.04)` | `var(--color-primary-5)` |
| `rgba(14, 165, 233, 0.05)` | `var(--color-primary-5)` |
| `rgba(14, 165, 233, 0.1)` | `var(--color-primary-10)` |
| `rgba(14, 165, 233, 0.2)` | `var(--color-primary-20)` |
| `rgba(14, 165, 233, 0.3)` | `var(--color-primary-30)` |
| `rgba(255, 255, 255, 0.8)` | `var(--color-bg-80)` |
| `rgba(255, 255, 255, 0.9)` | `var(--color-bg-90)` |
| `rgba(255, 255, 255, 0.95)` | `var(--color-bg-95)` |
| `rgba(226, 232, 240, 0.6)` | `var(--color-border-light)` |
| `rgba(226, 232, 240, 0.8)` | `var(--color-border-medium)` |
| `rgba(16, 185, 129, 0.1)` | `var(--color-success-10)` |
| `rgba(16, 185, 129, 0.3)` | `var(--color-success-30)` |
| `rgba(16, 185, 129, 0.4)` | `var(--color-success-40)` |
| `rgba(244, 63, 94, 0.02)` | `var(--color-error-2)` |
| `rgba(244, 63, 94, 0.1)` | `var(--color-error-10)` |
| `rgba(245, 158, 11, 0.1)` | `var(--color-warning-10)` |
| `rgba(0, 0, 0, 0.02)` | `var(--color-shadow)` |
| `rgba(0, 0, 0, 0.03)` | `var(--color-shadow)` |
| `rgba(0, 0, 0, 0.05)` | `var(--color-shadow-hover)` |
| `rgba(0, 0, 0, 0.08)` | `var(--color-shadow-md)` |
| `rgba(0, 0, 0, 0.1)` | `var(--color-shadow-lg)` |
| `rgba(0, 0, 0, 0.2)` | `var(--color-shadow-xl)` |
| `rgba(15, 23, 42, 0.1)` | `var(--color-scrim)` |
| `rgba(15, 23, 42, 0.25)` | `var(--color-scrim)` |
| `rgba(15, 23, 42, 0.4)` | `var(--color-scrim-dark)` |
| `rgba(148, 163, 184, 0.05)` | `var(--color-overlay)` |
| `rgba(148, 163, 184, 0.08)` | `var(--color-overlay-8)` |
