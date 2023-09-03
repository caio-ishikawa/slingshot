class Slingshot < Formula
  desc "Lightweight command line application with vim-like keybinds for quick file navigation."
  homepage "https://github.com/caio-ishikawa/slingshot"
  url "https://github.com/caio-ishikawa/slingshot/archive/refs/tags/v0.3.0.tar.gz"
  sha256 "65168f65612f82f2a0ed6faa7bf6e20fd141f5788219bcbd6c695ef92d2ea35b"

  depends_on "rust" => :build

  def install
    system "make", "install"
  end

  test do
    system "cargo", "test"
  end
end
