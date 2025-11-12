import SharedTypes
import SwiftUI

struct DownloadItem: View {
    @EnvironmentObject var core: Core
    var data: Download

    var body: some View {
        VStack(alignment: .leading) {
            Text(data.title)
            HStack(spacing: 12) {
                ProgressView(value: data.progress)
                Text("\(String(format: "%.0f", data.progress * 100))%")
            }
            if data.needs_file_mapping {
                Button {
                    core.navigationObserver?.push(screen: .serverFileMapping(data.id))
                } label: {
                    Label("Add File Mapping", systemImage: "map")
                }
            }
        }
    }
}

#Preview {
    Form {
        DownloadItem(
            data: Download(id: "24389729skjl", title: "Big Buck Bunny", progress: 0.6, needs_file_mapping: false, is_paused: false)
        )
        DownloadItem(
            data: Download(id: "24389729skjl", title: "Big Buck Bunny", progress: 0.6, needs_file_mapping: true, is_paused: false)
        )
    }
}
