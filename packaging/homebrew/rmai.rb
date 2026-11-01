# typed: false
# frozen_string_literal: true

# brew tap Ry3nZhao/Remove_CoAuthor https://github.com/Ry3nZhao/Remove_CoAuthor
# brew install rmai

class Rmai < Formula
  desc "Remove AI co-author trailers and related markers from Git history"
  homepage "https://github.com/Ry3nZhao/Remove_CoAuthor"
  version "0.1.2"
  license "MIT"
  head "https://github.com/Ry3nZhao/Remove_CoAuthor.git", branch: "main"

  depends_on "rust" => :build if build.head?

  on_macos do
    on_arm do
      url "https://github.com/Ry3nZhao/Remove_CoAuthor/releases/download/v0.1.2/rmai-aarch64-apple-darwin.tar.gz"
      sha256 "48bcf58c6f4b4841e048ff0e1e8142e50810ed021aebe9d6942af75976814ad7"
    end
    on_intel do
      url "https://github.com/Ry3nZhao/Remove_CoAuthor/releases/download/v0.1.2/rmai-x86_64-apple-darwin.tar.gz"
      sha256 "631fb4f8b1c5a426d78f22298e03a954d26825d37d154a49e0a4e7e4cc5d6fbd"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/Ry3nZhao/Remove_CoAuthor/releases/download/v0.1.2/rmai-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "f9208fe2e37699e9b9a7d4faea8f5f57f9639c8e2988e1384befb26d4d3fe11e"
    end
    on_intel do
      url "https://github.com/Ry3nZhao/Remove_CoAuthor/releases/download/v0.1.2/rmai-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "c2f7e92a76edb7488cf7845bf708add052ee8683ef75cab5f2c8d3033dbc6834"
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
