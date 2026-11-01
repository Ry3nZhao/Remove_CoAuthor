# rmai

[中文](README.zh-CN.md)

Remove AI co-author trailers and related markers (Claude, Codex, Cursor, Copilot, …) from Git history — **without changing any file content or tree**.

## Install

### Homebrew

```bash
brew tap Ry3nZhao/Remove_CoAuthor https://github.com/Ry3nZhao/Remove_CoAuthor
brew trust ry3nzhao/remove_coauthor   # required by newer Homebrew for third-party taps
brew install rmai
```

Note: third-party taps do not appear in default `brew search`; use the tap above. Install from source HEAD with `brew install --HEAD rmai` after tapping.

### Cargo / from source

```bash
cargo install --git https://github.com/Ry3nZhao/Remove_CoAuthor
# or
git clone https://github.com/Ry3nZhao/Remove_CoAuthor.git
cd Remove_CoAuthor && cargo install --path .
```

### Release binary

Download from [Releases](https://github.com/Ry3nZhao/Remove_CoAuthor/releases), or:

```bash
curl -fsSL https://raw.githubusercontent.com/Ry3nZhao/Remove_CoAuthor/main/scripts/install.sh | bash
```

## Usage

```bash
rmai --dry-run ./          # scan only
rmai clean ./              # or: rmai -c ./   (confirm)
rmai clean --yes ./        # or: rmai -C ./   (no confirm)

rmai no-more install ./
rmai no-more status
rmai no-more uninstall ./
rmai no-more install --all
rmai no-more uninstall --all

rmai whitelist add alice@example.com
rmai whitelist
rmai whitelist remove alice@example.com
```

After `clean`, SHAs change. Publish with:

```bash
git push --force-with-lease --all
git push --force-with-lease --tags
```

## Release

Push a version tag to build multi-arch binaries and publish a GitHub Release (also refreshes the Homebrew formula):

```bash
git tag v0.1.0
git push origin v0.1.0
```

## License

MIT
