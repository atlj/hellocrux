import SharedTypes
import SwiftUI

struct MediaManagerScreen: View {
    @EnvironmentObject var core: Core
    var overrideDownloads: [Download]?
    var overridePreparingList: [PreparingMedia]?
    var overrideMediaItems: MediaItems?

    var downloads: [Download] {
        overrideDownloads ?? core.view.downloads
    }

    var preparingList: [PreparingMedia] {
        overridePreparingList ?? core.view.preparing_list
    }

    var mediaItems: MediaItems {
        overrideMediaItems ?? core.view.media_items
    }

    var mediaValues: [Media]? {
        let vals: [String: Media].Values? =
            switch mediaItems {
            case let .loading(data):
                data?.values
            case let .success(data):
                data.values
            case .error:
                nil
            }

        guard let vals else {
            return nil
        }

        return Array(vals).sorted {
            $0.metadata.title < $1.metadata.title
        }
    }

    var body: some View {
        List {
            Section("Downloads") {
                NavigationLink(value: Screen.addDownload) {
                    Label("Download Media", systemImage: "square.and.arrow.down")
                }
                ForEach(downloads, id: \.id) { download in
                    DownloadItem(data: download)
                }
            }
            if !preparingList.isEmpty {
                Section("Preparing") {
                    ForEach(preparingList, id: \.hashValue) { preparingItem in
                        PreparingItem(data: preparingItem)
                    }
                }
            }
            Section {
                ForEach(mediaValues ?? [], id: \.id) { media in
                    NavigationLink(value: Screen.mediaManagerDetail(media)) {
                        ManageMediaItem(metadata: media.metadata)
                    }
                }
            } header: {
                HStack {
                    Text("Library")
                    if case .loading = core.view.media_items {
                        ProgressView()
                    }
                }
            } footer: {}
        }
        .navigationTitle("Manage Media")
        .task(priority: .background) {
            while true {
                core.update(.updateData(.getDownloads))
                core.update(.updateData(.getPreparing))
                do {
                    try await Task.sleep(for: .seconds(3))
                } catch {
                    return
                }
            }
        }
    }
}

#Preview {
    MediaManagerScreen(
        overrideDownloads: [
            Download(
                id: "sdlkfjvs", title: "Big Buck Bunny", progress: 0.2, needs_file_mapping: true,
                state: .inProgress,
            ),
            Download(
                id: "my movie", title: "My Movie", progress: 0.7, needs_file_mapping: true,
                state: .inProgress,
            ),
            Download(
                id: "skjvlk", title: "Skibbidy Toilet", progress: 0.0, needs_file_mapping: false,
                state: .paused,
            ),
        ],
        overridePreparingList: [
            .init(
                media: .movie(
                    id: "0",
                    path: .init(media: "Star Wars", track_name: "star_wars.mp4", subtitles: []),
                ),
                metadata: .init(
                    thumbnail:
                    "https://media.themoviedb.org/t/p/w440_and_h660_face/fai0rspsNeJCS69wHNjOdWxcI7P.jpg",
                    title: "Star Wars",
                ),
                needed_track_selection: nil,
            ),
            .init(
                media: .movie(
                    id: "0",
                    path: .init(media: "Star Wars", track_name: "star_wars.mp4", subtitles: []),
                ),
                metadata: .init(
                    thumbnail:
                    "https://media.themoviedb.org/t/p/w440_and_h660_face/fai0rspsNeJCS69wHNjOdWxcI7P.jpg",
                    title: "Star Wars",
                ),
                needed_track_selection: [],
            ),
            .init(
                media: .series(
                    id: "1", path: .init(media: "Breaking Bad", track_name: "br_ba", subtitles: []),
                    episode: .init(season_no: 3, episode_no: 10),
                ),
                metadata: .init(
                    thumbnail:
                    "https://media.themoviedb.org/t/p/w440_and_h660_face/anFx9aTOOYqgS3v7x3R84Kz67ly.jpg",
                    title: "Breaking Bad",
                ), needed_track_selection: nil,
            ),
            .init(
                media: .series(
                    id: "1", path: .init(media: "Breaking Bad", track_name: "br_ba", subtitles: []),
                    episode: .init(season_no: 3, episode_no: 10),
                ),
                metadata: .init(
                    thumbnail:
                    "https://media.themoviedb.org/t/p/w440_and_h660_face/anFx9aTOOYqgS3v7x3R84Kz67ly.jpg",
                    title: "Breaking Bad",
                ), needed_track_selection: [],
            ),

        ],
        overrideMediaItems: .success(data: [
            "Idiocracy": .init(
                id: "Idiocracy",
                metadata: .init(
                    thumbnail:
                    "https://www.themoviedb.org/t/p/w1280/k75tEyoPbPlfHSKakJBOR5dx1Dp.jpg",
                    title: "Idiocracy",
                ),
                content: .movie(.init(media: "", track_name: "", subtitles: [])),
            ),
        ]),
    ).environmentObject(Core.shared)
}
