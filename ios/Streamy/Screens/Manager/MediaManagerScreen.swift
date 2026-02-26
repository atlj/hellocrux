import SharedTypes
import SwiftUI

struct MediaManagerScreen: View {
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
            Section("Downloads") {
                NavigationLink(value: Screen.addDownload) {
                    Label("Download Media", systemImage: "square.and.arrow.down")
                }
                ForEach(downloads, id: \.id) { download in
                    DownloadItem(data: download)
                }
            }
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
        ]
    )
}
