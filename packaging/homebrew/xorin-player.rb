cask "xorin-player" do
  version "1.0.0"
  sha256 "6b4eeeb7ca307f131beff858a5e0fb08e70868f0dbf6b72b616edd7c080caf7e"

  url "https://github.com/shafayeatt1995/music_player/releases/download/v#{version}/xorin-player-macos-universal.dmg"
  name "Xorin Player"
  desc "Lightweight, cross-platform audio and cinema video player"
  homepage "https://github.com/shafayeatt1995/music_player"

  app "Xorin Player.app"
  binary "#{appdir}/Xorin Player.app/Contents/MacOS/xorin-player", target: "xorin-player"

  zap trash: [
    "~/.config/xorin-player",
    "~/Library/Application Support/xorin-player",
    "~/Library/Saved Application State/com.xorin.player.savedState",
  ]
end
