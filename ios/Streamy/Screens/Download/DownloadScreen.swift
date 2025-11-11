import SharedTypes
import SwiftUI

struct DownloadScreen: View {
    @EnvironmentObject var core: Core
    var overrideDownloads: [Download]?

    var downloads: [Download] {
        if let overrideDownloads {
            return overrideDownloads
        }

        return core.view.downloads
    }

    var body: some View {
        List {
            NavigationLink(value: Screen.addDownload) {
                Label("Add New Torrent", systemImage: "doc.fill.badge.plus")
            }
            Section {
                if downloads.isEmpty {
                    Text("No Downloads Yet")
                        .foregroundStyle(.gray)
                } else {
                    ForEach(downloads, id: \.id) { download in
                        DownloadItem(data: download)
                    }
                }
            }
        }
        .navigationTitle("Downloads")
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
    DownloadScreen(
        overrideDownloads: [
            Download(id: "sdlkfjvs", title: "Big Buck Bunny", progress: 0.2, is_paused: false),
            Download(id: "my movie", title: "My Movie", progress: 0.7, is_paused: false),
            Download(id: "skjvlk", title: "Skibbidy Toilet", progress: 0.0, is_paused: false),
        ]
    )
}
