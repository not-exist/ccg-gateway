# Skill 管理重构实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 重构 Skill 管理模块，使其符合设计文档规范

**Architecture:** 后端移除编辑仓库接口、重命名接口、修改重装流程；前端同步适配 API 调用

**Tech Stack:** Rust (Tauri commands), TypeScript (Vue 3 + Pinia)

---

## 文件结构

| 文件 | 职责 |
|------|------|
| `src-tauri/src/commands.rs` | 修改仓库/技能/收藏相关命令 |
| `src-tauri/src/services/skill.rs` | 新增批量 CLI 操作内部方法 |
| `frontend/src/api/skills.ts` | 适配新接口命名，移除旧方法 |
| `frontend/src/views/SkillManage.vue` | 移除编辑仓库功能，适配新接口 |

---

## Task 1: 移除编辑仓库接口（后端）

**Files:**
- Modify: `src-tauri/src/commands.rs:5358-5395` (移除 `update_skill_repo`)
- Modify: `src-tauri/src/services/skill.rs:209-227` (移除 `replace_skill_repo`)

- [ ] **Step 1: 删除 `update_skill_repo` 命令**

删除 `commands.rs` 第 5358-5395 行的 `update_skill_repo` 函数及相关 `sync_skill_favorites_repo` 调用。

- [ ] **Step 2: 删除 `replace_skill_repo` 函数**

删除 `skill.rs` 第 209-227 行的 `replace_skill_repo` 函数。

- [ ] **Step 3: 移除 lib.rs 中的命令注册**

在 `lib.rs` 中移除 `update_skill_repo` 的命令注册。

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/services/skill.rs src-tauri/src/lib.rs
git commit -m "refactor: remove update_skill_repo command"
```

---

## Task 2: 重命名仓库重装接口（后端）

**Files:**
- Modify: `src-tauri/src/commands.rs:5418-5436`

- [ ] **Step 1: 重命名 `refresh_repo_skills` 为 `reinstall_skill_repo`**

将第 5418 行函数名 `refresh_repo_skills` 改为 `reinstall_skill_repo`。

```rust
#[tauri::command]
pub async fn reinstall_skill_repo(
    db: State<'_, SqlitePool>,
    name: String,
) -> Result<Vec<DiscoverableSkill>> {
    // 保持原有逻辑不变
    let repo = skill::get_skill_repo(&name)?.ok_or_else(|| format!("未找到仓库 '{}'", name))?;

    let favorite_keys = get_skill_favorite_keys(db.inner()).await?;

    // 1. 本地目录
    if skill::is_local_repo_source(&repo.source) {
        return scan_local_repo_skills(&repo, &favorite_keys).await;
    }

    // 2. 远程仓库：删除缓存后重新 clone
    skill::delete_cached_repo_dir(&repo.source);
    let cache_dir = git_clone_repo(&repo.source)?;
    scan_cached_repo_skills(&cache_dir, &repo, &favorite_keys)
}
```

- [ ] **Step 2: 更新 lib.rs 命令注册**

将 `refresh_repo_skills` 改为 `reinstall_skill_repo`。

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "refactor: rename refresh_repo_skills to reinstall_skill_repo"
```

---

## Task 3: 重命名技能重装接口并修改流程（后端）

**Files:**
- Modify: `src-tauri/src/commands.rs:5637-5684`

- [ ] **Step 1: 新增 `batch_set_skill_cli` 内部方法**

在 `commands.rs` 中 `remove_skill_from_all_cli_async` 函数后新增：

```rust
// 批量设置 CLI 启用状态（内部方法）
async fn batch_set_skill_cli(db: &SqlitePool, directory: &str, cli_types: &[String]) -> Result<()> {
    // 先从所有 CLI 移除
    remove_skill_from_all_cli_async(db, directory).await?;

    // 再启用指定的 CLI
    for cli_type in cli_types {
        sync_skill_to_cli_async(db, directory, cli_type).await?;
    }
    Ok(())
}
```

- [ ] **Step 2: 新增 `detect_skill_cli_status` 内部方法**

```rust
// 检测技能在各 CLI 的启用状态（遍历文件系统）
async fn detect_skill_cli_status(db: &SqlitePool, directory: &str) -> Vec<String> {
    let mut enabled_clis = Vec::new();
    for cli_type in ["claude_code", "codex", "gemini"] {
        if skill_enabled_in_cli_async(db, cli_type, directory).await {
            enabled_clis.push(cli_type.to_string());
        }
    }
    enabled_clis
}
```

- [ ] **Step 3: 重写 `reinstall_skill` 函数**

重命名并修改流程：

```rust
#[tauri::command]
pub async fn reinstall_skill(
    db: State<'_, SqlitePool>,
    directory: String,
) -> Result<InstalledSkillResponse> {
    // 1. 检测当前 CLI 启用状态
    let enabled_clis = detect_skill_cli_status(db.inner(), &directory).await;

    // 2. 从 manifest 获取信息
    let manifest_entries = skill::load_installed_skill_manifest()?;
    let entry = manifest_entries
        .iter()
        .find(|e| e.directory == directory)
        .ok_or_else(|| format!("Skill '{}' not found in manifest", directory))?;

    let repo = entry
        .repo
        .as_ref()
        .ok_or_else(|| "Skill missing repo info, cannot reinstall".to_string())?;

    let source_dir = entry
        .source_directory
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or_else(|| {
            if skill::is_local_repo_source(&repo.source) && directory == repo.name {
                "."
            } else {
                &directory
            }
        });

    // 3. 删除 SSOT 目录
    let ssot_dir = get_ssot_dir();
    let skill_path = ssot_dir.join(&directory);
    if skill_path.exists() {
        std::fs::remove_dir_all(&skill_path).map_err(|e| e.to_string())?;
    }

    // 4. 从仓库源复制到 SSOT
    let skill_source_path = if skill::is_local_repo_source(&repo.source) {
        let base = std::path::Path::new(&repo.source);
        if source_dir == "." { base.to_path_buf() } else { base.join(source_dir) }
    } else {
        let cache_dir = git_clone_repo(&repo.source)?;
        if source_dir == "." { cache_dir.to_path_buf() } else { cache_dir.join(source_dir) }
    };

    if !skill_source_path.exists() {
        return Err(format!("技能目录不存在: {}", skill_source_path.display()));
    }
    copy_dir_recursive(&skill_source_path, &skill_path)?;

    // 5. 恢复 CLI 启用状态
    batch_set_skill_cli(db.inner(), &directory, &enabled_clis).await?;

    // 6. 返回结果
    let cli_flags = build_skill_cli_flags(db.inner(), &directory).await;
    let (disk_name, disk_description) = read_installed_skill_metadata(&directory);
    let key = format!("{}:{}", repo.name, source_dir);

    Ok(InstalledSkillResponse {
        id: directory.clone(),
        name: disk_name.unwrap_or_else(|| entry.name.clone()),
        description: normalize_skill_text(&disk_description.unwrap_or_default()),
        directory,
        repo: Some(repo.clone()),
        readme_url: None,
        installed_at: entry.installed_at,
        cli_flags,
        exists_on_disk: true,
        is_favorited: false,
        can_favorite: true,
        favorite_key: Some(key),
        market_display: if is_local_repo_source(&repo.source) {
            String::new()
        } else {
            format!("@{}", repo.source)
        },
    })
}
```

- [ ] **Step 4: 删除旧的 `reinstall_installed_skill` 函数**

删除第 5637-5684 行的旧函数。

- [ ] **Step 5: 更新 lib.rs 命令注册**

将 `reinstall_installed_skill` 改为 `reinstall_skill`。

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "refactor: rewrite reinstall_skill with status detection flow"
```

---

## Task 4: 修改 `install_skill` 移除 CLI 同步逻辑（后端）

**Files:**
- Modify: `src-tauri/src/commands.rs:5535-5626`

- [ ] **Step 1: 修改 `install_skill_inner` 移除 CLI 同步**

安装技能时不触发 CLI 同步，CLI 状态由用户手动启用：

```rust
async fn install_skill_inner(
    db: &SqlitePool,
    skill_item: DiscoverableSkill,
    reinstall: bool,
) -> Result<InstalledSkillResponse> {
    let ssot_dir = get_ssot_dir();
    let directory_name = skill_install_directory_name(&skill_item);
    let skill_path = ssot_dir.join(&directory_name);
    let existing = skill::load_installed_skill_manifest()?
        .into_iter()
        .find(|entry| entry.directory == directory_name);

    skill::ensure_repo_exists(&skill_item.repo)?;

    if (existing.is_some() || skill_path.exists()) && !reinstall {
        return Err(format!("Skill '{}' is already installed", directory_name));
    }

    // 如果是重装，先删除旧的 SSOT 目录
    if reinstall && skill_path.exists() {
        let _ = std::fs::remove_dir_all(&skill_path);
    }

    // 根据类型进行安装
    if skill::is_local_repo_source(&skill_item.repo.source) {
        let source_path = std::path::Path::new(&skill_item.repo.source);
        let skill_source_path = if skill_item.directory == "." {
            source_path.to_path_buf()
        } else {
            source_path.join(&skill_item.directory)
        };

        let dest_path = ssot_dir.join(&directory_name);
        if !skill_source_path.exists() {
            return Err(format!("技能目录不存在: {}", skill_source_path.display()));
        }

        copy_dir_recursive(&skill_source_path, &dest_path)?;
    } else {
        // 从 git clone 缓存目录复制
        let cache_dir = git_clone_repo(&skill_item.repo.source)?;
        let skill_source_path = if skill_item.directory == "." {
            cache_dir.to_path_buf()
        } else {
            cache_dir.join(&skill_item.directory)
        };

        let dest_path = ssot_dir.join(&directory_name);
        if !skill_source_path.exists() {
            return Err(format!("技能目录不存在: {}", skill_source_path.display()));
        }

        copy_dir_recursive(&skill_source_path, &dest_path)?;
    }

    let now = chrono::Utc::now().timestamp();
    skill::upsert_installed_skill_manifest_entry(InstalledSkillManifestEntry {
        directory: directory_name.clone(),
        name: directory_name.clone(),
        description: None,
        repo: Some(skill_item.repo.clone()),
        readme_url: None,
        installed_at: now,
        source_directory: Some(skill_item.directory.clone()),
    })?;

    // 不触发 CLI 同步，状态实时检测
    let cli_flags = build_skill_cli_flags(db, &directory_name).await;

    Ok(InstalledSkillResponse {
        id: directory_name.clone(),
        name: skill_item.name,
        description: normalize_skill_text(&skill_item.description),
        directory: directory_name,
        repo: Some(skill_item.repo.clone()),
        readme_url: skill_item.readme_url,
        installed_at: now,
        cli_flags,
        exists_on_disk: true,
        is_favorited: skill_item.is_favorited,
        can_favorite: true,
        favorite_key: Some(skill_item.key.clone()),
        market_display: if is_local_repo_source(&skill_item.repo.source) {
            String::new()
        } else {
            format!("@{}", skill_item.repo.source)
        },
    })
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/commands.rs
git commit -m "refactor: remove CLI sync from install_skill_inner"
```

---

## Task 5: 新增从收藏重装接口（后端）

**Files:**
- Modify: `src-tauri/src/commands.rs`

- [ ] **Step 1: 新增 `reinstall_favorite_skill` 命令**

在 `install_favorite_skill` 后新增：

```rust
#[tauri::command]
pub async fn reinstall_favorite_skill(
    db: State<'_, SqlitePool>,
    key: String,
) -> Result<InstalledSkillResponse> {
    let favorite =
        sqlx::query_as::<_, SkillFavorite>("SELECT * FROM skill_favorites WHERE skill_key = ?")
            .bind(&key)
            .fetch_optional(db.inner())
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Skill favorite not found".to_string())?;

    // 计算安装目录
    let directory = skill_install_directory_name_from_parts(&favorite.directory, &favorite.repo_name);

    // 检查是否已安装
    let ssot_dir = get_ssot_dir();
    if !ssot_dir.join(&directory).exists() {
        return Err(format!("Skill '{}' is not installed, cannot reinstall", directory));
    }

    // 调用 reinstall_skill
    reinstall_skill(db.inner(), directory).await
}
```

- [ ] **Step 2: 更新 lib.rs 命令注册**

添加 `reinstall_favorite_skill` 注册。

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat: add reinstall_favorite_skill command"
```

---

## Task 6: 修改从收藏安装逻辑（后端）

**Files:**
- Modify: `src-tauri/src/commands.rs:5849-5922`

- [ ] **Step 1: 修改 `install_favorite_skill` 添加仓库检查**

```rust
#[tauri::command]
pub async fn install_favorite_skill(
    db: State<'_, SqlitePool>,
    key: String,
) -> Result<InstalledSkillResponse> {
    let favorite =
        sqlx::query_as::<_, SkillFavorite>("SELECT * FROM skill_favorites WHERE skill_key = ?")
            .bind(&key)
            .fetch_optional(db.inner())
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Skill favorite not found".to_string())?;

    let repo = SkillRepo {
        name: favorite.repo_name.clone(),
        source: favorite.repo_source.clone(),
    };

    // 检查仓库是否已安装，未安装则静默安装
    let existing_repo = skill::get_skill_repo(&repo.name)?;
    if existing_repo.is_none() {
        // 静默安装仓库
        if skill::is_local_repo_source(&repo.source) {
            let path = std::path::Path::new(&repo.source);
            if !path.exists() || !path.is_dir() {
                return Err(format!("本地目录 {} 不存在", repo.source));
            }
            skill::upsert_skill_repo(repo.clone())?;
        } else {
            git_clone_repo(&repo.source)?;
            skill::upsert_skill_repo(repo.clone())?;
        }
    }

    // 确保仓库缓存存在
    let cache_dir = if skill::is_local_repo_source(&repo.source) {
        std::path::Path::new(&repo.source).to_path_buf()
    } else {
        git_clone_repo(&repo.source)?
    };

    // 后续逻辑保持不变...
    let skill_path = if favorite.directory == "." {
        cache_dir.to_path_buf()
    } else {
        cache_dir.join(&favorite.directory)
    };

    let (directory, skill_key) = if skill_path.exists() {
        (favorite.directory.clone(), favorite.skill_key.clone())
    } else {
        let skills = scan_cached_repo_skills(&cache_dir, &repo, &std::collections::HashSet::new())?;
        let skill = skills
            .iter()
            .find(|s| s.name == favorite.name)
            .ok_or_else(|| format!("未在仓库中找到技能: {}", favorite.name))?;

        sqlx::query("UPDATE skill_favorites SET directory = ?, skill_key = ? WHERE skill_key = ?")
            .bind(&skill.directory)
            .bind(&skill.key)
            .bind(&favorite.skill_key)
            .execute(db.inner())
            .await
            .map_err(map_db_error)?;

        (skill.directory.clone(), skill.key.clone())
    };

    install_skill_inner(
        db.inner(),
        DiscoverableSkill {
            key: skill_key,
            name: favorite.name,
            description: favorite.description.unwrap_or_default(),
            directory: directory.clone(),
            install_directory: skill_install_directory_name_from_parts(&directory, &favorite.repo_name),
            readme_url: favorite.readme_url,
            repo,
            is_favorited: true,
            is_installed: false,
        },
        false,
    )
    .await
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/commands.rs
git commit -m "refactor: add repo check in install_favorite_skill"
```

---

## Task 7: 前端 API 适配

**Files:**
- Modify: `frontend/src/api/skills.ts`
- Modify: `frontend/src/types/models.ts` (如有新增类型)

- [ ] **Step 1: 移除 `updateRepo` 方法**

删除第 38-40 行：
```typescript
updateRepo: async (oldName: string, newUrl: string): Promise<SkillRepo> => {
  return await invoke<SkillRepo>('update_skill_repo', { oldName, newUrl })
},
```

- [ ] **Step 2: 重命名 `refreshRepoSkills` 为 `reinstallRepo`**

```typescript
reinstallRepo: async (name: string): Promise<DiscoverableSkill[]> => {
  return await invoke<DiscoverableSkill[]>('reinstall_skill_repo', { name })
},
```

- [ ] **Step 3: 重命名 `reinstallInstalled` 为 `reinstall`**

```typescript
reinstall: async (directory: string): Promise<InstalledSkill> => {
  const result = await invoke<InstalledSkillBackend>('reinstall_skill', { directory })
  return transformInstalledSkill(result)
},
```

- [ ] **Step 4: 新增 `reinstallFavorite` 方法**

```typescript
reinstallFavorite: async (key: string): Promise<InstalledSkill> => {
  const result = await invoke<InstalledSkillBackend>('reinstall_favorite_skill', { key })
  return transformInstalledSkill(result)
},
```

- [ ] **Step 5: Commit**

```bash
git add frontend/src/api/skills.ts
git commit -m "refactor: adapt frontend API to new skill commands"
```

---

## Task 8: 前端页面适配

**Files:**
- Modify: `frontend/src/views/SkillManage.vue` (或其他使用旧 API 的页面)
- Modify: `frontend/src/stores/` (如有使用旧 API 的 store)

- [ ] **Step 1: 移除编辑仓库功能**

找到编辑仓库的按钮/对话框，移除相关代码和 `updateRepo` 调用。

- [ ] **Step 2: 更新重装仓库调用**

将 `refreshRepoSkills` 改为 `reinstallRepo`。

- [ ] **Step 3: 更新重装技能调用**

将 `reinstallInstalled` 改为 `reinstall`。

- [ ] **Step 4: 新增从收藏重装功能**

在收藏列表中添加重装按钮，调用 `reinstallFavorite`。

- [ ] **Step 5: Commit**

```bash
git add frontend/src/views/ frontend/src/stores/
git commit -m "refactor: update frontend pages for skill management"
```

---

## Task 9: 验证测试

- [ ] **Step 1: 运行后端编译检查**

```bash
cd src-tauri && cargo check
```
Expected: 无编译错误

- [ ] **Step 2: 运行前端类型检查**

```bash
cd frontend && pnpm type-check
```
Expected: 无类型错误

- [ ] **Step 3: 功能验证**

手动测试以下场景：
1. 安装仓库 → 删除仓库（验证连带卸载技能）
2. 重装仓库（远程）
3. 安装技能 → CLI 启用/停用 → 重装技能（验证状态恢复）
4. 从收藏安装（仓库未安装时）
5. 从收藏重装

- [ ] **Step 4: Commit (如有修复)**

```bash
git add .
git commit -m "fix: resolve compilation and type errors"
```

---

## 自检清单

### Spec Coverage

| 设计要求 | 对应 Task |
|----------|-----------|
| 移除编辑仓库 | Task 1 |
| 重命名 reinstall_skill_repo | Task 2 |
| 重命名 reinstall_skill | Task 3 |
| 重装技能：检测状态 → 删除 → 复制 → 恢复 | Task 3 |
| 安装技能不触发 CLI 同步 | Task 4 |
| 新增 reinstall_favorite_skill | Task 5 |
| 从收藏安装：检查仓库 | Task 6 |
| 前端 API 适配 | Task 7 |
| 前端页面适配 | Task 8 |

### Placeholder Scan

无 TBD/TODO 占位符，所有代码步骤包含完整实现。

### Type Consistency

- `InstalledSkillResponse` 类型保持不变
- `DiscoverableSkill` 类型保持不变
- 前端 `InstalledSkill.cli_flags` 格式保持 `Record<string, boolean>`