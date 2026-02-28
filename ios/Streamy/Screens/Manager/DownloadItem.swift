import SharedTypes
import SwiftUI

struct DownloadItem: View {
    @EnvironmentObject var core: Core
    var data: Download

    var body: some View {
        VStack(alignment: .leading) {
            Text(data.title)
            DownloadStateBadge(state: data.state)
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

struct DownloadStateBadge: View {
    var state: DownloadState

    var title: String {
        switch state {
        case .paused:
            "Paused"
        case .failed:
            "Failed"
        case .inProgress:
            "Downloading"
        case .processing:
            "Processing"
        case .complete:
            "Completed"
        }
    }

    var color: Color {
        switch state {
        case .paused:
            .gray
        case .failed:
            .red
        case .inProgress:
            .blue
        case .processing:
            .orange
        case .complete:
            .green
        }
    }

    var body: some View {
        HStack {
            Text(title)
                .font(.callout)
                .padding(6)
                .background(RoundedRectangle(cornerSize: .init(width: 6, height: 6)).fill(color).opacity(0.3))
        }
    }
}

#Preview("Badge") {
    VStack {
        DownloadStateBadge(state: .complete)
        DownloadStateBadge(state: .failed)
        DownloadStateBadge(state: .inProgress)
        DownloadStateBadge(state: .paused)
        DownloadStateBadge(state: .processing)
    }
}

#Preview {
    Form {
        DownloadItem(
            data: Download(id: "24389729skjl", title: "Big Buck Bunny", progress: 0.6, needs_file_mapping: false, state: .inProgress)
        )
        DownloadItem(
            data: Download(id: "24389729skjl", title: "Big Buck Bunny", progress: 1.0, needs_file_mapping: true, state: .complete)
        )
    }
}
