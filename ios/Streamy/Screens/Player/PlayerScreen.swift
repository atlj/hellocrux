import AVKit
import SharedTypes
import SwiftUI

struct PlayerScreen: View {
    let url: URL
    private let player: AVPlayer
    let itemId: String
    let episode: Episode?

    init(url: URL, itemId: String, episode: Episode? = nil, initialSeconds: UInt64?) {
        self.url = url
        player = AVPlayer(url: url)
        player.play()

        if let initialSeconds {
            player.seek(to: .init(seconds: Double(initialSeconds), preferredTimescale: CMTimeScale(NSEC_PER_SEC)))
        }

        self.itemId = itemId
        self.episode = episode

        player.addPeriodicTimeObserver(forInterval: .init(seconds: 1, preferredTimescale: CMTimeScale(NSEC_PER_SEC)), queue: .main) { time in
            if time.seconds.rounded(.towardZero) == 0 {
                return
            }

            Core.shared.update(.playbackProgress(.init(id: itemId, episode: episode, progress_seconds: UInt64(time.seconds))))
        }
    }

    @EnvironmentObject var core: Core

    var body: some View {
        Player(player: player) { seconds in
            if seconds == 0 {
                return
            }

            core.update(.playbackProgress(.init(id: itemId, episode: episode, progress_seconds: seconds)))
        }
    }
}

#Preview {
    PlayerScreen(
        url: URL(string: "http://localhost:3000/static/jaho/recording.mov")!,
        itemId: "1", initialSeconds: nil
    )
    .environmentObject(Core())
}
