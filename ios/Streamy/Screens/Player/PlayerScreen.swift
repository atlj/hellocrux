import AVKit
import SharedTypes
import SwiftUI

struct PlayerScreen: View {
    @EnvironmentObject var core: Core
    var overrideData: ActivePlayerData?

    var data: ActivePlayerData? {
        overrideData ?? core.view.playback_detail.active_player
    }

    var title: String {
        var title = data!.metadata.title

        if let episode_id = data?.current_episode {
            title += " \(episode_id)"
        }

        return title
    }

    var body: some View {
        if data != nil {
            Player(data: data!) { duration, position in
                Core.shared.update(.playbackProgress(.init(duration, position)))
            }
            .onAppear {
                core.update(.screenChanged(.player))
            }
            .navigationTitle(title)
        }
    }
}

#Preview {
    PlayerScreen(
        overrideData: .init(id: "id", position: .movie(id: "1", position_seconds: 0), media_paths: .init(media: "", track_name: "", subtitles: []), metadata: .init(thumbnail: "", title: "Test", tmdb_id: nil), current_episode: nil, next_episode: nil),
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

extension SharedTypes.EpisodeIdentifier: @retroactive CustomStringConvertible {
    public var description: String {
        "S\(season_no)E\(episode_no)"
    }
}
