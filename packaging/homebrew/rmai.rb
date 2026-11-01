# typed: false
# frozen_string_literal: true

# Placeholder / HEAD formula. Binary URLs are filled by release CI.
#   brew tap Ry3nZhao/Remove_CoAuthor https://github.com/Ry3nZhao/Remove_CoAuthor
#   brew install rmai

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
