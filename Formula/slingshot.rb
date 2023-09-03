class Slingshot < Formula
  desc "Lightweight command line application with vim-like keybinds for quick file navigation."
  homepage "https://github.com/caio-ishikawa/slingshot"
  url "https://github.com/caio-ishikawa/slingshot/archive/refs/tags/v0.3.0.tar.gz"
  sha256 "00938ab8ff104bcc9796253c1d046de0b1d45588d5a81100d6f414be73d863c8"

  depends_on "rust" => :build

  def install
    system "make", "install"
  end

  test do
    system "cargo", "test"
  end
end
