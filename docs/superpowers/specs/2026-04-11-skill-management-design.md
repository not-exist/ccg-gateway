# Skill 管理逻辑设计

## 概述

本文档定义 CCG Gateway 项目中 Skill 管理模块的业务逻辑规范，用于指导后端接口重构和前端适配。

---

## 一、核心概念

| 概念 | 说明 |
|------|------|
| 仓库 (Repo) | 技能的来源，可以是本地目录或远程 Git 仓库 |
| SSOT 目录 | `data/skills/`，技能安装后的唯一真实来源 |
| 缓存目录 | `data/skills/.cache/`，远程仓库 clone 的存放位置 |
| CLI 技能目录 | 各 CLI 工具的 skills 目录，从 SSOT 复制 |

---

## 二、仓库管理

### 安装仓库

```
输入: source (本地路径或远程 URL)

流程:
  本地源: 验证路径存在 → 记录到 config.json
  远程源: git clone 到 .cache → 记录到 config.json

输出: 仓库信息
```

### 重装仓库

```
输入: source

流程:
  本地源: 无操作
  远程源: 删除 .cache 缓存 → 重新 clone

输出: 更新后的仓库信息
```

### 删除仓库

```
输入: source

流程:
  1. 卸载该仓库下所有已安装技能
  2. 远程源: 删除缓存
  3. 从 config.json 移除

输出: 无
```

---

## 三、技能管理

### 安装技能

```
输入: skill 信息（包含仓库源、技能目录）

流程:
  1. 从仓库源（本地或缓存）复制技能到 SSOT 目录
  2. 写入 manifest（不存储 CLI 状态）

输出: 已安装技能信息
```

### 重装技能

```
输入: skill id (directory)

流程:
  1. 遍历所有 CLI 目录，检测该技能的启用状态（临时变量）
  2. 删除 SSOT 目录
  3. 从仓库源复制到 SSOT
  4. 按检测到的状态恢复到各 CLI

输出: 已安装技能信息
```

### 卸载技能

```
输入: skill id (directory)

流程:
  1. 遍历所有 CLI 目录，删除该技能
  2. 删除 SSOT 目录
  3. 从 manifest 移除

输出: 无
```

---

## 四、CLI 管理

### 启用/停用

```
启用: 复制 SSOT 目录到 CLI skills 目录
停用: 删除 CLI skills 目录中的技能
```

### 接口设计

**对外 API**:
```
toggle_skill_cli(id, cliType, enabled) → 切换单个 CLI 的启用状态
```

**内部方法**:
```
batch_set_skill_cli(id, cliTypes) → 批量设置 CLI 启用状态
  - 供卸载技能、重装技能内部调用
  - 不暴露为独立 API
```

---

## 五、收藏管理

### 收藏技能

```
输入: skill 信息

流程:
  写入数据库（skill_key = repo_name:directory 作为唯一标识）

输出: 无
```

### 取消收藏

```
输入: skill_key

流程:
  从数据库删除

输出: 无
```

### 从收藏安装

```
输入: skill_key

流程:
  1. 从数据库获取收藏信息
  2. 检查仓库是否已安装
     - 未安装: 静默安装仓库
  3. 触发安装技能

输出: 已安装技能信息
```

### 从收藏重装

```
输入: skill_key

流程:
  触发重装技能

输出: 已安装技能信息
```

---

## 六、缓存管理

- **策略**: 按需清理
- **触发时机**:
  - 重装仓库时（远程源）
  - 删除仓库时（远程源）
- **不提供**: 定期清理、手动清理所有缓存

---

## 七、错误处理

- 操作失败不回滚，保留已完成步骤
- 返回错误信息，用户可重试
- 无前置校验

---

## 八、接口清单

### 仓库管理

| 接口 | 说明 |
|------|------|
| `get_skill_repos` | 获取仓库列表 |
| `add_skill_repo` | 安装仓库 |
| `reinstall_skill_repo` | 重装仓库 |
| `remove_skill_repo` | 删除仓库 |

### 技能管理

| 接口 | 说明 |
|------|------|
| `discover_repo_skills` | 发现仓库中的技能 |
| `install_skill` | 安装技能 |
| `reinstall_skill` | 重装技能 |
| `uninstall_skill` | 卸载技能 |
| `get_installed_skills` | 获取已安装技能列表 |

### CLI 管理

| 接口 | 说明 |
|------|------|
| `toggle_skill_cli` | 切换单个 CLI 启用状态 |
| `get_cli_list` | 获取已配置 CLI 列表 |

### 收藏管理

| 接口 | 说明 |
|------|------|
| `get_skill_favorites` | 获取收藏列表 |
| `add_skill_favorite` | 收藏技能 |
| `remove_skill_favorite` | 取消收藏 |
| `install_favorite_skill` | 从收藏安装 |
| `reinstall_favorite_skill` | 从收藏重装 |

---

## 九、数据模型

### 仓库配置 (config.json)

```json
{
  "repos": [
    {
      "source": "/path/to/local/repo",
      "name": "local-repo"
    },
    {
      "source": "https://github.com/user/repo.git",
      "name": "repo"
    }
  ],
  "installed_skills": [
    {
      "directory": "skill-name",
      "repo_source": "https://github.com/user/repo.git",
      "source_directory": "skills/skill-name",
      "installed_at": 1712800000
    }
  ]
}
```

### 收藏数据 (数据库)

```sql
skill_favorites (
  id INTEGER PRIMARY KEY,
  skill_key TEXT UNIQUE,  -- repo_name:directory
  name TEXT,
  description TEXT,
  directory TEXT,
  readme_url TEXT,
  repo_name TEXT,
  repo_source TEXT,
  repo_branch TEXT,
  created_at INTEGER
)
```

---

## 十、与现有实现的差异

| 功能点 | 现有实现 | 新设计 | 原因 |
|--------|----------|--------|------|
| 编辑仓库 | `update_skill_repo` | 移除 | 简化操作，要么新增要么删除 |
| CLI 状态存储 | manifest 中存 `cli_flags` | 不存储，实时检测 | 状态应从文件系统实时读取 |
| 批量 CLI API | 无独立接口 | 内部方法 | 仅供卸载/重装调用，不暴露 |
| 重装技能流程 | 直接覆盖 + sync | 检测状态 → 删除 → 复制 → 恢复 | 确保状态一致性 |
| 从收藏安装 | 不检查仓库 | 先检查仓库 | 避免仓库不存在时失败 |