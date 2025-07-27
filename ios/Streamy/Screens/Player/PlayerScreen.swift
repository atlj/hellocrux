import AVKit
import SharedTypes
import SwiftUI

struct PlayerScreen: View {
    @EnvironmentObject var core: Core

    var body: some View {
        if let playerData = core.view.playback_detail.active_player {
            Player(data: playerData) { duration, position in
                Core.shared.update(.playbackProgress(.init(duration, position)))
            }
            .navigationTitle(playerData.title)
        }
    }
}

#Preview {
    PlayerScreen()
        .environmentObject(Core())
}

extension SharedTypes.PlaybackPosition {
    func getInitialSeconds() -> UInt64 {
        switch self {
        case .movie(id: _, position_seconds: let position_seconds):
            position_seconds
        case .seriesEpisode(id: _, episode_identifier: _, position_seconds: let position_seconds):
            position_seconds
        }
    }
}
