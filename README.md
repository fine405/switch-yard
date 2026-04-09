# Switchyard

`Switchyard` 是一个用统一栈重新开始的 `codex-auth` 桌面版重构项目。

一期目标只做一件事：先把 mac 菜单栏应用的开发、打包、分发链路跑通，同时保留未来扩展到 Windows / Linux / Terminal CLI 的基础。

`Switchyard` 不是 `Loongphy/codex-auth` 或 OpenAI 的官方项目。

## 技术栈

- 桌面壳：Tauri 2
- 前端：React 19 + TypeScript + Vite
- 共享核心：Rust workspace
- 菜单栏定位：`tauri-plugin-positioner`
- 打包产物：`.app` + `.dmg`

这一版采用单一技术栈思路：

- mac / Windows / Linux 桌面都走 `Tauri + React + Rust`
- 未来 CLI 直接复用 `src-tauri/crates/switchyard-core`

## 一期范围

当前已经复刻 `codex-auth` 的核心切换能力：

- 读取 `~/.codex/accounts/registry.json`
- 读取账号快照 `~/.codex/accounts/*.auth.json`
- 展示账号列表、当前激活账号、套餐与用量概览
- 切换激活账号并回写 `~/.codex/auth.json`
- 切换 `auto_switch.enabled`
- 切换 `api.usage`
- 监听 `registry.json` / `auth.json` 变化并实时刷新面板

当前暂不包含：

- 登录导入
- 账号删除
- 系统服务管理
- Windows / Linux 外壳
- 独立 CLI 入口

## 目录结构

```text
switchyard/
├─ src/                          React 菜单栏面板
├─ src-tauri/
│  ├─ src/                       Tauri 壳与托盘逻辑
│  └─ crates/switchyard-core/    共享 Rust 核心
└─ package.json
```

## 本地开发

先安装依赖：

```bash
pnpm install
```

启动 mac 菜单栏开发模式：

```bash
pnpm dev:desktop
```

默认读取当前用户的 `~/.codex`。如果你要对测试数据演练，可以覆盖：

```bash
export SWITCHYARD_CODEX_HOME=/path/to/mock/.codex
pnpm dev:desktop
```

## 打包与分发

本地生成 mac 安装包：

```bash
pnpm build:mac
```

产物路径：

- `src-tauri/target/release/bundle/macos/Switchyard.app`
- `src-tauri/target/release/bundle/dmg/Switchyard_0.1.0_aarch64.dmg`

如果你只是想快速做本机分发验证，可以用 ad-hoc 签名：

```bash
pnpm build:mac:adhoc
```

这会等价于：

```bash
APPLE_SIGNING_IDENTITY=- pnpm tauri build --bundles app,dmg
```

注意：ad-hoc 签名只适合本机或受控环境验证。没有 notarization 的情况下，Gatekeeper 仍会拒绝外部分发包。

如果要发给外部用户，建议补齐正式签名和 notarization，再执行：

```bash
pnpm build:mac
```

常见准备项：

- 本机钥匙串中已经安装 Apple Developer 证书
- 设置 `APPLE_SIGNING_IDENTITY`
- 设置 notarization 所需的 Apple 凭据环境变量

Tauri 官方分发文档：

- https://v2.tauri.app/distribute/
- https://v2.tauri.app/distribute/sign/macos/

## 验证命令

我在当前项目上已跑通以下检查：

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --all
cargo test --manifest-path src-tauri/Cargo.toml -p switchyard-core
cargo check --manifest-path src-tauri/Cargo.toml
pnpm build
pnpm build:mac
```

## 后续路线

二期建议按这个顺序推进：

1. 抽出 CLI 入口，复用 `switchyard-core`
2. 增加登录导入 / 账号管理
3. 补 Windows 托盘窗口
4. 补 Linux 托盘窗口
5. 增加自动更新与正式签名发布流水线

## 许可证与来源说明

本项目当前采用 MIT 协议发布，见 `LICENSE`。

`Switchyard` 是独立命名、独立实现的桌面项目，但一期功能范围、兼容的数据文件结构，以及部分行为设计参考了 `Loongphy/codex-auth`。我已补充第三方声明，保留其 MIT 许可证与版权信息，见 `THIRD_PARTY_NOTICES.md`。

为避免品牌和来源混淆：

- 本项目不使用 `codex-auth` 作为产品名
- README 中明确声明非上游官方项目
- `Codex`、`OpenAI`、`ChatGPT` 等名称仅用于兼容性说明，其商标权归各自权利人所有
