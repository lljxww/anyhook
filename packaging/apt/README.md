# Linux APT 仓库发布指南

因为我们在 `cargo-deb` 中已经配置好了原生的 `.deb` 生成流程，所以现在把 Anyhook 发布到 APT 源供 `apt-get install anyhook` 使用非常简单。

## 方案 1: 使用 GitHub Actions 结合第三方托管 (推荐)

最现代化、免运维的方案是使用包托管服务，例如 **Cloudsmith**, **Gemfury**, 或 **Aptly**。
以 Cloudsmith 为例，你可以在刚才的 `release.yml` 流水线中，在生成 `.deb` 的步骤之后添加自动上传脚本：

```yaml
      - name: Push to Cloudsmith APT Repo
        run: |
          pip install cloudsmith-cli
          cloudsmith push deb lljxww/anyhook-repo/ubuntu/jammy target/debian/*.deb
        env:
          CLOUDSMITH_API_KEY: ${{ secrets.CLOUDSMITH_API_KEY }}
```

用户安装时只需：
```bash
curl -1sLf 'https://dl.cloudsmith.io/public/lljxww/anyhook/setup.deb.sh' | sudo -E bash
sudo apt-get install anyhook
```

## 方案 2: 使用 Ubuntu Launchpad PPA

如果你想提交到 Ubuntu 官方的 PPA：
1. 你的源码根目录下需要有一个规范的 `debian/` 目录（包含 `control`, `rules`, `changelog` 等）。
2. PPA 平台**不接受**预编译好的二进制文件。因此在 `debian/rules` 里必须写明 `cargo build --release` 进行现编。
3. 相比之下，方案 1 更适合 Rust 编写的独立二进制软件。

## 方案 3: 自建 APT 服务器

如果你想部署在公司内网：
1. 在一台 Linux 服务器上安装 `aptly` (APT Repository Management tool)
2. 将 GitHub Actions 产出的 `.deb` 包推送到该服务器，并使用 aptly 导入：
   ```bash
   aptly repo add anyhook-repo ./anyhook_1.0.0_amd64.deb
   aptly publish repo anyhook-repo
   ```
3. 通过 Nginx 将 `~/.aptly/public` 目录暴露为静态 HTTP 站点。
