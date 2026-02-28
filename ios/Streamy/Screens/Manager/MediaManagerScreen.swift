import SharedTypes
import SwiftUI

struct MediaManagerScreen: View {
    @EnvironmentObject var core: Core
    var overrideDownloads: [Download]?
    var overrideMediaItems: MediaItems?

    var downloads: [Download] {
        overrideDownloads ?? core.view.downloads
    }

    var mediaItems: MediaItems {
        overrideMediaItems ?? core.view.media_items
    }

    var mediaValues: [Media]? {
        let vals: [String: Media].Values? = switch mediaItems {
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
            Section {
                ForEach(mediaValues ?? [], id: \.id) { media in
                    ManageMediaItem(metadata: media.metadata)
                }
            } header: {
                HStack {
                    Text("Media")
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
                do {
                    try await Task.sleep(for: .seconds(5))
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
            Download(id: "sdlkfjvs", title: "Big Buck Bunny", progress: 0.2, needs_file_mapping: true, state: .inProgress),
            Download(id: "my movie", title: "My Movie", progress: 0.7, needs_file_mapping: true, state: .inProgress),
            Download(id: "skjvlk", title: "Skibbidy Toilet", progress: 0.0, needs_file_mapping: false, state: .paused),
        ], overrideMediaItems: .success(data: ["Idiocracy": .init(id: "Idiocracy", metadata: .init(thumbnail: "https://www.themoviedb.org/t/p/w1280/k75tEyoPbPlfHSKakJBOR5dx1Dp.jpg", title: "Idiocracy"), content: .movie(.init(media: "", subtitles: [])))])
    ).environmentObject(Core.shared)
}
