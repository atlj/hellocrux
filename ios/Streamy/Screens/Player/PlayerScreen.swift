import AVKit
import SharedTypes
import SwiftUI

struct PlayerScreen: View {
    let url: URL
    let itemId: String
    let episode: Episode?
    let initialSeconds: UInt64?

    @EnvironmentObject var core: Core

    var body: some View {
        Player(url: url, initialSeconds: initialSeconds) { time in

            Core.shared.update(.playbackProgress(.init(id: itemId, episode: episode, progress_seconds: UInt64(time.seconds))))
        }
    }
}

#Preview {
    PlayerScreen(
        url: URL(string: "http://localhost:3000/static/jaho/recording.mov")!,
        itemId: "1", episode: nil, initialSeconds: nil
    )
    .environmentObject(Core())
}
