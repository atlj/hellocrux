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
                .frame(minWidth: 40, alignment: .top)

                VStack(alignment: .leading, spacing: 0) {
                    VStack(alignment: .leading) {
                        Text(data.metadata.title)
                        if let episodeId {
                            Text("S\(episodeId.season_no) E\(episodeId.episode_no)")
                                .font(.callout)
                                .foregroundStyle(.secondary)
                        }
                    }

                    if case let .encoding(progress: progress, remaining_seconds: remaining_seconds) = data.state {
                        VStack(alignment: .leading, spacing: 0) {
                            HStack(spacing: 12) {
                                ProgressView(value: progress)
                                HStack {
                                    Text("\(String(format: "%.0f", progress * 100))%")
                                        .foregroundStyle(.secondary)
                                }
                            }

                            let remainingDuration = Swift.Duration.seconds(remaining_seconds)
                            Text("Encoding: \(remainingDuration.formatted(.units(width: .abbreviated))) left...")
                                .foregroundStyle(.secondary)
                        }
                        .padding(.top, 2)
                    }

                    if case .extractingSubtitles = data.state {
                        Text("Extracting subtitles...")
                            .foregroundStyle(.secondary)
                            .padding(.top, 4)
                    }

                    if case .moving = data.state {
                        Text("Moving media files...")
                            .foregroundStyle(.secondary)
                            .padding(.top, 4)
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
                state: .waiting, media: .movie(
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
                state: .encoding(progress: 0.73, remaining_seconds: 5000), media: .movie(
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
                state: .extractingSubtitles, media: .movie(
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
                state: .moving, media: .movie(
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
                state: .waiting, media: .movie(
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
                state: .waiting, media: .series(
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
                state: .encoding(progress: 0.31, remaining_seconds: 832), media: .series(
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
                state: .extractingSubtitles, media: .series(
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
                state: .moving, media: .series(
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
                state: .waiting, media: .series(
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
