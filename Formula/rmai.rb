# typed: false
# frozen_string_literal: true

# Placeholder until the first tagged release fills in real sha256 values.
# Install (tap this repo):
#   brew tap Ry3nZhao/Remove_CoAuthor https://github.com/Ry3nZhao/Remove_CoAuthor
#   brew install rmai
#
# Or build from source before the first binary release:
#   brew install --HEAD Ry3nZhao/Remove_CoAuthor/rmai

class Rmai < Formula
  desc "Remove AI co-author trailers and related markers from Git history"
  homepage "https://github.com/Ry3nZhao/Remove_CoAuthor"
  license "MIT"
  head "https://github.com/Ry3nZhao/Remove_CoAuthor.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "rmai", shell_output("#{bin}/rmai --version")
  end
end
