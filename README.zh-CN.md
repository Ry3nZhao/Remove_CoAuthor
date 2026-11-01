# rmai

[English](README.md)

清除 Git 历史与新提交中的 AI co-author / 相关标注（Claude、Codex、Cursor、Copilot 等），**不改动任何文件内容与目录结构**。

## 安装

### Homebrew

```bash
brew tap Ry3nZhao/Remove_CoAuthor https://github.com/Ry3nZhao/Remove_CoAuthor
brew trust ry3nzhao/remove_coauthor   # 新版 Homebrew 安装第三方 tap 需要先信任
brew install rmai
```

说明：第三方 tap 不会出现在默认的 `brew search` 结果里，需按上面方式添加 tap。源码 HEAD 安装：tap 之后执行 `brew install --HEAD rmai`。

### Cargo / 源码

```bash
cargo install --git https://github.com/Ry3nZhao/Remove_CoAuthor
# 或
git clone https://github.com/Ry3nZhao/Remove_CoAuthor.git
cd Remove_CoAuthor && cargo install --path .
```

### Release 二进制

从 [Releases](https://github.com/Ry3nZhao/Remove_CoAuthor/releases) 下载，或：

```bash
curl -fsSL https://raw.githubusercontent.com/Ry3nZhao/Remove_CoAuthor/main/scripts/install.sh | bash
```

## 用法

```bash
rmai --dry-run ./          # 只扫描
rmai clean ./              # 或 rmai -c ./   （需确认）
rmai clean --yes ./        # 或 rmai -C ./   （跳过确认）

rmai no-more install ./
rmai no-more status
rmai no-more uninstall ./
rmai no-more install --all
rmai no-more uninstall --all

rmai whitelist add alice@example.com
rmai whitelist
rmai whitelist remove alice@example.com
```

`clean` 会改写 SHA，发布时请自行：

```bash
git push --force-with-lease --all
git push --force-with-lease --tags
```

## 发布

推送版本 tag 后，CI 会构建多架构二进制并创建 GitHub Release（同时更新 Homebrew Formula）：

```bash
git tag v0.1.0
git push origin v0.1.0
```

## 许可证

MIT
