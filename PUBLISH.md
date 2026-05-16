# 把 `omnivideo-sdk` 发布到 crates.io

## 前置条件

1. 装好 Rust 工具链：

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
   source "$HOME/.cargo/env"
   ```

2. 用 GitHub 账号到 <https://crates.io/> 登录（首次会要求授权 GitHub OAuth）。
3. 在 <https://crates.io/settings/tokens> 新建 **API Token**：
   - **Name**: `omnivideo-publish`
   - **Scopes**: 勾 `publish-new` + `publish-update`（首次发包必须有 publish-new，已发的包更新只要 publish-update）
   - 复制下来的字符串就是 `CARGO_REGISTRY_TOKEN`，**只显示一次**。
4. 包名 `omnivideo-sdk` 不能被人占用。检查 <https://crates.io/crates/omnivideo-sdk>。

## 发布步骤

```bash
cd rust
cargo login <CARGO_REGISTRY_TOKEN>      # 一次性,凭证存到 ~/.cargo/credentials.toml
cargo publish --dry-run                 # 检查能否打包
cargo publish                           # 上传到 crates.io
```

也可以用环境变量免交互（CI 友好）：

```bash
CARGO_REGISTRY_TOKEN=cio_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx \
  cargo publish
```

## 后续版本升级

修改 `Cargo.toml` 里 `version`，重新 `cargo publish` 即可。**已发布的版本不能被覆盖**（可以 yank，但不推荐除非有安全问题）。
