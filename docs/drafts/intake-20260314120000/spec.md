---
title: "Spec: Codex Backend Support"
doc_type: proc
brief: "Design spec for adding OpenAI Codex CLI backend to cct TUI launcher"
confidence: verified
created: 2026-03-14
updated: 2026-03-14
revision: 1
source_skill: idea
---

# Spec: Codex Backend Support

## Chosen Approach

**Unified Profile struct + runtime validation** (方案 A). Profile struct 增加 `backend: Backend` 枚举字段（默认 Claude）和 `full_auto: Option<bool>`（codex only）。通过 `validate()` 在解析后检查字段互斥性，而非编译时类型约束。

**Rationale**: 只有 2 个 backend、各自 1 个独有字段，运行时校验足够。避免 serde/toml tagged enum 的反序列化复杂度。符合 KISS 原则。

## Architecture Decisions

### 1. Config (src/config.rs)

- 新增 `Backend` 枚举: `Claude` (default) | `Codex`，`#[serde(rename_all = "lowercase")]`
- Profile struct 增加 `backend: Backend` (`#[serde(default)]`) 和 `full_auto: Option<bool>`
- `load_profiles()` 后调用 `validate()`:
  - Codex + `skip_permissions = Some(true)` → error
  - Claude + `full_auto = Some(true)` → error
- `NewProfile` 增加 `backend: Backend` 和 `full_auto: Option<bool>` 字段
- `append_profile()` 根据 backend 生成不同的 env block（codex 用 `OPENAI_API_KEY`）
- 向后兼容：无 `backend` 字段 → 默认 Claude，无 `full_auto` → None

### 2. App State (src/app.rs)

- `App` 增加 `active_backend: Backend` 字段
- Tab/1/2 键切换 `active_backend`，切换后 `selected` 重置为目标 backend 第一个 profile
- `next()`/`prev()` 只在当前 backend 的 profile 子集内导航
- `FormState` 增加 `backend: Backend`，表单字段根据 backend 动态决定
- Claude form: Name, Description, Base URL, API Key, Model (5 字段)
- Codex form: Name, Description, API Key, Model, Full Auto (5 字段)

### 3. UI (src/ui.rs)

- Profile list 标题下增加 tab 指示器 `[Claude] [Codex]`，当前 tab 高亮
- 列表只显示当前 backend 的 profiles
- Detail panel 对 codex profile 显示 `full_auto` 而非 `skip_permissions`
- Footer 增加 `[Tab/1/2] Backend` 提示
- Add form 根据 backend 显示不同字段标签

### 4. Launch (src/launch.rs)

- 新增 `build_codex_args(profile: &Profile) -> Vec<String>`: `--model`, `--full-auto`, extra_args
- 新增 `exec_codex(profile: &Profile) -> anyhow::Error`: 注入 env + exec "codex"
- codex 不支持 `--continue`，`c` 键（resume）对 codex profile 无效

### 5. main.rs

- Enter 键根据 `profile.backend` 分发到 `exec_claude`/`exec_codex`
- `c` 键只对 Claude profile 生效
- Tab/1/2 键切换 active_backend
- `a` 键根据 active_backend 创建对应 backend 的 FormState
- `s` 键（toggle skip_permissions）只对 Claude profile 生效

## Out of Scope

- Codex auto-install（二进制缺失时不提示安装）
- Codex `--sandbox`, `--ask-for-approval` 等高级 flags
- Generic backend trait abstraction
- Codex resume 支持
- Model 字段 env-var 自动生成（codex 用自己的配置系统）

## Acceptance Criteria

- [ ] AC1: `profiles.toml` 含 codex profile（backend = "codex"）可正确解析
- [ ] AC2: 非法字段组合在 parse 时被拒绝（codex+skip_permissions, claude+full_auto）
- [ ] AC3: TUI 显示 tab/group 分隔 claude 和 codex profiles
- [ ] AC4: 选中 codex profile 按 Enter exec-replace 为 `codex <args>`
- [ ] AC5: `full_auto = true` 产生 `codex --full-auto`
- [ ] AC6: Codex env vars 从 `[profiles.env]` 注入 exec 环境
- [ ] AC7: codex config 解析和 arg building 的单元测试通过
- [ ] AC8: 集成测试覆盖 create-display-launch flow
- [ ] AC9: 现有 claude-only profiles 无需修改即可工作
