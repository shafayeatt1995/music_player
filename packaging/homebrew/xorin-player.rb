cask "xorin-player" do
  version "1.0.0"
  sha256 "79469f630412e0d9fa96e2e97851fea92aa13bcd68d7d3d798aa8be00807be3f"

  url "https://github.com/shafayeatt1995/music_player/releases/download/v#{version}/xorin-player-macos-universal.dmg"
  name "Xorin Player"
  desc "Lightweight, cross-platform audio and cinema video player"
  homepage "https://github.com/shafayeatt1995/music_player"

  app "Xorin Player.app"
  binary "#{appdir}/Xorin Player.app/Contents/MacOS/xorin-player", target: "xorin-player"

  postflight do
    system_command "/usr/bin/xattr",
                   args: ["-cr", "#{appdir}/Xorin Player.app"]
  end

  zap trash: [
    "~/.config/xorin-player",
    "~/Library/Application Support/xorin-player",
    "~/Library/Saved Application State/com.xorin.player.savedState",
  ]
end
