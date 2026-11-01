# typed: false
# frozen_string_literal: true

# brew tap Ry3nZhao/Remove_CoAuthor https://github.com/Ry3nZhao/Remove_CoAuthor
# brew install rmai

class Rmai < Formula
  desc "Remove AI co-author trailers and related markers from Git history"
  homepage "https://github.com/Ry3nZhao/Remove_CoAuthor"
  version "0.1.1"
  license "MIT"
  head "https://github.com/Ry3nZhao/Remove_CoAuthor.git", branch: "main"

  depends_on "rust" => :build if build.head?

  on_macos do
    on_arm do
      url "https://github.com/Ry3nZhao/Remove_CoAuthor/releases/download/v0.1.1/rmai-aarch64-apple-darwin.tar.gz"
      sha256 "d3cbefa3b32d69f338b63ee3aff6b66b0ab9ab35e96e03049520bd2c506d621d"
    end
    on_intel do
      url "https://github.com/Ry3nZhao/Remove_CoAuthor/releases/download/v0.1.1/rmai-x86_64-apple-darwin.tar.gz"
      sha256 "e617642d469b15fb1cd7830ebc56937a43245691c09d2e0c971575668bbff336"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/Ry3nZhao/Remove_CoAuthor/releases/download/v0.1.1/rmai-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "867f13f930887ae480b4f70358adfed9dcb0afaea3c1bcbee33d2519a4bdfe61"
    end
    on_intel do
      url "https://github.com/Ry3nZhao/Remove_CoAuthor/releases/download/v0.1.1/rmai-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "91d8fb82a4065ba60807d7f3d9b92dcb79ccd961d19ef0f420b87eaa31f0bcea"
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
