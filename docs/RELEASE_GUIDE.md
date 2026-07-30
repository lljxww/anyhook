# Anyhook 发布与分发指南

本指南将教你如何将 Anyhook 编译、打包并分发给其他用户。项目目前支持**本地手动打包**和**GitHub Actions 全自动流水线**两种方式。

## 1. 方式一：GitHub Actions 全自动化发布 (推荐) 🚀

如果你将 Anyhook 托管在 GitHub，系统已经内置了标准的 Release 流水线 (`.github/workflows/release.yml`)。你不需要在本地配置任何复杂的交叉编译环境。

### 操作步骤：
1. **修改版本号**:
   进入 `crates/anyhook-cli/Cargo.toml` 以及根目录的工作空间配置，把 `version = "x.y.z"` 升级到你想要发布的版本号。
2. **提交代码并打标签 (Tag)**:
   ```bash
   git add .
   git commit -m "chore: release v1.0.0"
   git tag v1.0.0
   git push origin main --tags
   ```
3. **等待云端构建**:
   推送 tag 后，前往 GitHub 的 **Actions** 标签页，你会看到名为 `Release` 的工作流正在运行。
   它会在后台并行启动 Windows、macOS 和 Linux 服务器，全自动帮你交叉编译出各种平台所需的文件。
4. **获取产物**:
   等流水线跑完，去代码仓库右侧的 **Releases** 区域，你会看到一个全新的发布版！
   里面包含了：
   - 🍏 macOS (Intel & Apple Silicon) 的 `.tar.gz`
   - 🪟 Windows 的 `.exe` 和 `.zip`
   - 🐧 Linux (x86 & ARM64) 的 `.tar.gz`
   - 🐧 **Linux 专用的 `.deb` Debian/Ubuntu 无脑安装包**

---

## 2. 方式二：本地一键打包脚本 📦

如果你只是想快速在当前电脑上生成一个可以直接发给同事或者测试环境的归档包，可以使用根目录提供的 `package.sh` 脚本。

### 操作步骤：
1. **运行脚本**:
   在终端中执行：
   ```bash
   ./package.sh
   ```
2. **检查产出**:
   脚本会自动执行 `cargo build --release`。编译完成后，它会在根目录下创建一个 `dist/` 文件夹。
   在该文件夹中，你可以找到一个干净的、已经剥离源码的压缩包（macOS/Linux 是 `.tar.gz`，Windows 环境下是 `.zip`），里面包含：
   - `anyhook` (二进制主程序文件)
   - `anyhook.yaml.sample` (配置模板)
   - `README.md` (说明文档)
   - `LICENSE` (开源协议)
3. **分发**:
   直接把 `dist/` 里的压缩包发给目标机器，对方解压后只需执行 `./anyhook start` 即可运行。

### 本地打包 `.deb` (限 Linux 开发者)
如果你当前使用的是 Linux 系统，并想在本地打一个 `.deb` 包：
1. 请先安装 `cargo-deb` 扩展工具：
   ```bash
   cargo install cargo-deb
   ```
2. 然后再次运行 `./package.sh`，脚本会自动侦测到 `cargo-deb` 并为你生成适用于 Ubuntu/Debian 的原装 `.deb` 安装包。

---

## 3. 高级：Linux 交叉编译说明
如果你不使用 GitHub Actions，想要在 Mac 或 Windows 上本地为 Linux 编译二进制文件。我们强烈建议使用 [cross](https://github.com/cross-rs/cross) 工具（它底层基于 Docker）：

```bash
cargo install cross
cross build --release --target x86_64-unknown-linux-gnu -p anyhook-cli
```
编译产物会在 `target/x86_64-unknown-linux-gnu/release/` 下。
