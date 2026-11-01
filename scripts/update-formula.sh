#!/usr/bin/env bash
# Update Formula/rmai.rb from release asset SHA256 files in dist/
set -euo pipefail

VER="${1:?version required, e.g. 0.1.0}"
DIST_DIR="${2:-dist}"
BASE="https://github.com/Ry3nZhao/Remove_CoAuthor/releases/download/v${VER}"

sha() {
  awk '{print $1}' "${DIST_DIR}/rmai-$1.tar.gz.sha256"
}

SHA_ARM=$(sha aarch64-apple-darwin)
SHA_X64=$(sha x86_64-apple-darwin)
SHA_L64=$(sha x86_64-unknown-linux-gnu)
SHA_LARM=$(sha aarch64-unknown-linux-gnu)

mkdir -p Formula packaging/homebrew

cat > Formula/rmai.rb <<EOF
# typed: false
# frozen_string_literal: true

# brew tap Ry3nZhao/Remove_CoAuthor https://github.com/Ry3nZhao/Remove_CoAuthor
# brew install rmai

class Rmai < Formula
  desc "Remove AI co-author trailers and related markers from Git history"
  homepage "https://github.com/Ry3nZhao/Remove_CoAuthor"
  version "${VER}"
  license "MIT"
  head "https://github.com/Ry3nZhao/Remove_CoAuthor.git", branch: "main"

  depends_on "rust" => :build if build.head?

  on_macos do
    on_arm do
      url "${BASE}/rmai-aarch64-apple-darwin.tar.gz"
      sha256 "${SHA_ARM}"
    end
    on_intel do
      url "${BASE}/rmai-x86_64-apple-darwin.tar.gz"
      sha256 "${SHA_X64}"
    end
  end

  on_linux do
    on_arm do
      url "${BASE}/rmai-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "${SHA_LARM}"
    end
    on_intel do
      url "${BASE}/rmai-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "${SHA_L64}"
    end
  end

  def install
    if build.head?
      system "cargo", "install", *std_cargo_args
    else
      bin.install "rmai"
    end
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/rmai --version")
  end
end
EOF

cp Formula/rmai.rb packaging/homebrew/rmai.rb
echo "Updated Formula/rmai.rb for v${VER}"
