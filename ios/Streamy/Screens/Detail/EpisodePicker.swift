import SharedTypes
import SwiftUI

struct EpisodePicker: View {
    let id: String
    let series: [UInt32: [UInt32: MediaPaths]]
    @State private var season = 1
    @EnvironmentObject var core: Core

    var data: [Season] {
        var seasons = [Season]()

        for (seasonNumber, episodes) in series {
            var children = [Season]()
            for (episodeNumber, _) in episodes {
                children.append(Season(data: .episode(Int(seasonNumber), Int(episodeNumber))))
            }
            children.sort { $0.number < $1.number }
            seasons.append(Season(data: .season(Int(seasonNumber)), children: children))
        }

        seasons.sort { $0.number < $1.number }
        return seasons
    }

    var body: some View {
        OutlineGroup(data, children: \.children) { item in
            switch item.data {
            case let .season(seasonId):
                Text("Season \(seasonId)")
                    .font(.title3)
            case let .episode(seasonId, episodeId):
                Button {
                    core.update(.play(.fromCertainEpisode(id: id, episode: .init(season_no: UInt32(seasonId), episode_no: UInt32(episodeId)))))
                } label: {
                    HStack {
                        Text("Episode \(episodeId)")
                        Spacer()
                    }
                }
                .accessibilityLabel("Episode \(episodeId), Season \(seasonId)")
            }
        }
        .listRowBackground(Color.clear)
    }
}

#Preview {
    VStack {
        EpisodePicker(
            id: "test", series: [
                1: [
                    8: .init(media: "", subtitles: []),
                    1: .init(media: "", subtitles: []),
                    2: .init(media: "", subtitles: []),
                ],
                2: [:],
            ]
        )
        .environmentObject(Core())
    }
}

struct Season: Hashable, Identifiable {
    var id: Self { self }
    var data: SeasonData
    var children: [Season]? = nil

    var number: Int {
        switch data {
        case let .season(seasonNumber):
            seasonNumber
        case let .episode(_, episodeNumber):
            episodeNumber
        }
    }
}

enum SeasonData: Hashable {
    case season(Int)
    case episode(Int, Int)
}
