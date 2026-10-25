import SharedTypes
import SwiftUI

struct PreparingItem: View {
    @EnvironmentObject var core: Core
    var data: PreparingMedia

    var episodeId: EpisodeIdentifier? {
        guard case .series(id: _, path: _, episode: let episodeIdentifier) = data.media else {
            return nil
        }
        return episodeIdentifier
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                AsyncImage(url: URL(string: data.metadata.thumbnail)) { image in
                    image
                        .resizable()
                        .scaledToFit()
                        .clipShape(RoundedRectangle(cornerRadius: 4.0))
                } placeholder: {
                    ProgressView()
                }
                .frame(height: 60)
                .frame(minWidth: 40)
                VStack(alignment: .leading) {
                    Text(data.metadata.title)
                    if let episodeId {
                        Text("S\(episodeId.season_no) E\(episodeId.episode_no)")
                            .font(.callout)
                            .foregroundStyle(.secondary)
                    }
                }
            }

            if data.needed_track_selection != nil {
                NavigationLink(value: Screen.trackSelection(preparing_media: data)) {
                    Label("Select Tracks", systemImage: "movieclapper")
                }
            }
        }
    }
}

#Preview {
    List {
        PreparingItem(
            data: .init(
                media: .movie(
                    id: "0",
                    path: .init(media: "Star Wars", track_name: "star_wars.mp4", subtitles: []),
                ),
                metadata: .init(
                    thumbnail:
                    "https://media.themoviedb.org/t/p/w440_and_h660_face/fai0rspsNeJCS69wHNjOdWxcI7P.jpg",
                    title: "Star Wars", tmdb_id: nil,
                ), needed_track_selection: nil,
            ),
        )
        PreparingItem(
            data: .init(
                media: .movie(
                    id: "0",
                    path: .init(media: "Star Wars", track_name: "star_wars.mp4", subtitles: []),
                ),
                metadata: .init(
                    thumbnail:
                    "https://media.themoviedb.org/t/p/w440_and_h660_face/fai0rspsNeJCS69wHNjOdWxcI7P.jpg",
                    title: "Star Wars", tmdb_id: nil,
                ), needed_track_selection: [],
            ),
        )
        PreparingItem(
            data: .init(
                media: .series(
                    id: "1", path: .init(media: "", track_name: "", subtitles: []),
                    episode: .init(season_no: 3, episode_no: 10),
                ),
                metadata: .init(
                    thumbnail:
                    "https://media.themoviedb.org/t/p/w440_and_h660_face/anFx9aTOOYqgS3v7x3R84Kz67ly.jpg",
                    title: "Breaking Bad", tmdb_id: nil,
                ), needed_track_selection: nil,
            ),
        )
        PreparingItem(
            data: .init(
                media: .series(
                    id: "1", path: .init(media: "", track_name: "", subtitles: []),
                    episode: .init(season_no: 3, episode_no: 10),
                ),
                metadata: .init(
                    thumbnail:
                    "https://media.themoviedb.org/t/p/w440_and_h660_face/anFx9aTOOYqgS3v7x3R84Kz67ly.jpg",
                    title: "Breaking Bad", tmdb_id: nil,
                ), needed_track_selection: [],
            ),
        )
    }
}
