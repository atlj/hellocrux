import AVKit
import SharedTypes
import SwiftUI

struct PlayerScreen: View {
    @EnvironmentObject var core: Core
    var overrideData: ActivePlayerData?

    var data: ActivePlayerData? {
        overrideData ?? core.view.playback_detail.active_player
    }

    var body: some View {
        if data != nil {
            Player(data: data!) { duration, position in
                Core.shared.update(.playbackProgress(.init(duration, position)))
            }
            .onAppear {
                core.update(.screenChanged(.player))
            }
            .navigationTitle(data!.title)
        }
    }
}

#Preview {
    PlayerScreen(
        overrideData: .init(position: .movie(id: "1", position_seconds: 0), url: "http://localhost:3000/static/jaho/recording.mov", title: "Test", next_episode: nil)
    )
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

    func getId() -> String {
        switch self {
        case let .movie(id: id, position_seconds: _):
            id
        case let .seriesEpisode(id: id, episode_identifier: _, position_seconds: _):
            id
        }
    }
}
