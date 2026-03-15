---
title: "Intake Session Log"
doc_type: proc
status: activated
brief: "Q&A transcript for codex support in cct"
confidence: speculative
created: 2026-03-14
updated: 2026-03-15
revision: 2
context_ref: "docs/references/context-20260314120000/"
---

# Intake Session Log

**Session**: intake-20260314120000
**Status**: active

## Context Snapshot

- **Project**: cct — Rust TUI launcher for Claude Code CLI
- **Tech stack**: Rust, ratatui, serde/toml, toml_edit, cargo test, bats-core
- **Existing modules**: config, app, ui, launch, cli (5-module flat architecture)
- **Prior sessions**: 6 intake sessions, all activated
- **Source**: GitHub Issue #6 "Codex Support" — 3 points: auto-install, new UI, codex-specific flags

## Q&A Transcript

### Round 1
**Category**: Goal
**Question**: Issue #6 提到的 'codex' 指的是 OpenAI 的 Codex CLI 工具（openai/codex）吗？你希望 cct 同时支持启动 claude 和 codex 两种 CLI 工具？
**Answer**: 是，支持 OpenAI Codex CLI
**Files Read**: GitHub Issue #6

### Round 2
**Category**: Constraints
**Question**: codex 和 claude 的 profile 应该如何组织？你偏好哪种方式？
**Answer**: 单配置文件 + backend 字段（推荐方案）
**Files Read**: None

### Round 3
**Category**: Existing State
**Question**: 你目前对 OpenAI Codex CLI 的了解程度如何？
**Answer**: 用户提供了完整的 `codex -h` 输出，包含所有子命令和选项。关键参数：`--model`, `--sandbox`, `--ask-for-approval`, `--full-auto`, `--search`, `--cd`, `--profile` 等。
**Files Read**: None (user-provided CLI help output)

### Round 4
**Category**: Scope
**Question**: Issue #6 提到了三个点，你希望第一版包含哪些？
**Answer**: codex 专属 UI, codex flags 映射, profile backend 字段（不含 codex 自动安装）
**Files Read**: None

### Round 5
**Category**: Scope (detail)
**Question**: 你希望 codex profile 支持哪些 codex 特有的配置字段？
**Answer**: full-auto（仅此一项）
**Files Read**: None

### Round 6
**Category**: Constraints (detail)
**Question**: profile 的 backend 字段应该默认为 'claude' 还是强制要求用户指定？
**Answer**: 一个配置文件，支持两套 backend 独立的 profile，各个 backend 支持字段检查
**Files Read**: None

### Round 7
**Category**: Scope (UI)
**Question**: TUI 中如何区分显示 claude 和 codex 的 profile？
**Answer**: 分组/Tab 切换
**Files Read**: None

### Round 8
**Category**: Acceptance
**Question**: 你希望的验收标准是什么？
**Answer**: E2E + 单元测试
**Files Read**: None

### Round 9 (resumed session)
**Category**: Constraints (technical detail)
**Question**: codex 的启动正确路径是什么？
**Answer**: 用户提供了完整启动机制：
1. 设置 `CODEX_HOME` 为 `~/.config/cct-tui/codex/`
2. 启动命令：`OPENAI_API_KEY="<从profile注入>" CODEX_HOME=~/.config/cct-tui/codex/ codex [--full-auto]`
3. 在启动前生成 `~/.config/cct-tui/codex/config.toml`：
   ```toml
   model_provider = "custom"
   model = "gpt-5.4"
   [model_providers.custom]
   name = "<从profile注入name>"
   base_url = "<从profile注入url>"
   ```
**Files Read**: None (user-provided)

### Round 10
**Category**: Constraints (technical detail)
**Question**: 多个 codex profile 使用不同 model/base_url 时，config.toml 如何处理？
**Answer**: 共用一个 config.toml，启动前重写内容
**Files Read**: None

### Round 11
**Category**: Constraints (technical detail)
**Question**: codex config.toml 是 cct 自动生成还是用户手动维护？
**Answer**: cct 从 profile 字段自动生成
**Files Read**: None

### Round 12
**Category**: Scope (detail)
**Question**: codex 启动参数是否需要支持 --full-auto 以外的参数？
**Answer**: 仅 full_auto
**Files Read**: None

## Summary
**Rounds**: 12
**Stop Reason**: All categories answered with confidence (resumed session)
**Gaps**: None
