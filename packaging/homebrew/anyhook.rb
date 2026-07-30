class Anyhook < Formula
  desc "A cross-platform event-driven automation engine"
  homepage "https://github.com/lljxww/anyhook"
  version "1.0.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/lljxww/anyhook/releases/download/v#{version}/anyhook-macos-aarch64.tar.gz"
      # sha256 "REPLACE_WITH_ACTUAL_SHA256"
    else
      url "https://github.com/lljxww/anyhook/releases/download/v#{version}/anyhook-macos-x86_64.tar.gz"
      # sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/lljxww/anyhook/releases/download/v#{version}/anyhook-linux-aarch64.tar.gz"
      # sha256 "REPLACE_WITH_ACTUAL_SHA256"
    else
      url "https://github.com/lljxww/anyhook/releases/download/v#{version}/anyhook-linux-x86_64.tar.gz"
      # sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
  end

  def install
    bin.install "anyhook"
    
    # Install sample configuration if present
    if File.exist?("anyhook.yaml.sample")
      etc.install "anyhook.yaml.sample" => "anyhook/anyhook.yaml"
    end
  end

  # Setup Launchd to run anyhook as a background service
  service do
    run [opt_bin/"anyhook", "start", "-c", etc/"anyhook/anyhook.yaml"]
    keep_alive true
    log_path var/"log/anyhook.log"
    error_log_path var/"log/anyhook.error.log"
  end

  test do
    system "#{bin}/anyhook", "--help"
  end
end
