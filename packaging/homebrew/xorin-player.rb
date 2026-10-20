cask "xorin-player" do
  version "1.0.0"
  sha256 :no_check

  url "https://github.com/shafayeatt1995/music_player/releases/download/v#{version}/xorin-player-macos-app.zip"
  name "Xorin Player"
  desc "Lightweight, cross-platform, VLC-grade audio & cinema video player"
  homepage "https://github.com/shafayeatt1995/music_player"

  app "Xorin Player.app"
  binary "#{appdir}/Xorin Player.app/Contents/MacOS/xorin-player", target: "xorin-player"

  zap trash: [
    "~/.config/xorin-player",
    "~/Library/Application Support/xorin-player",
    "~/Library/Saved Application State/com.xorin.player.savedState",
  ]
end
