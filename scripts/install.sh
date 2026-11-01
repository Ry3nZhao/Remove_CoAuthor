#!/usr/bin/env bash
# Install rmai from GitHub Releases into ~/.local/bin (override with PREFIX).
set -euo pipefail

REPO="${RMAI_REPO:-Ry3nZhao/Remove_CoAuthor}"
VERSION="${RMAI_VERSION:-latest}"
PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="${PREFIX}/bin"

os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"
case "$arch" in
  x86_64|amd64) arch="x86_64" ;;
  arm64|aarch64) arch="aarch64" ;;
  *) echo "unsupported arch: $arch" >&2; exit 1 ;;
esac

case "$os" in
  darwin) target="${arch}-apple-darwin" ;;
  linux) target="${arch}-unknown-linux-gnu" ;;
  *) echo "unsupported os: $os" >&2; exit 1 ;;
esac

if [[ "$VERSION" == "latest" ]]; then
  url="https://github.com/${REPO}/releases/latest/download/rmai-${target}.tar.gz"
else
  url="https://github.com/${REPO}/releases/download/v${VERSION}/rmai-${target}.tar.gz"
fi

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

echo "Downloading $url"
curl -fsSL "$url" -o "$tmpdir/rmai.tar.gz"
tar -xzf "$tmpdir/rmai.tar.gz" -C "$tmpdir"
bin_path="$(find "$tmpdir" -type f -name rmai | head -n1)"
if [[ -z "$bin_path" ]]; then
  echo "rmai binary not found in archive" >&2
  exit 1
fi

mkdir -p "$BIN_DIR"
install -m 0755 "$bin_path" "$BIN_DIR/rmai"
echo "Installed $BIN_DIR/rmai"
echo "Ensure $BIN_DIR is on your PATH"
