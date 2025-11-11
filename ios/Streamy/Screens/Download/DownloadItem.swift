import SharedTypes
import SwiftUI

struct DownloadItem: View {
    var data: Download

    var body: some View {
        VStack(alignment: .leading) {
            Text(data.title)
            HStack(spacing: 12) {
                ProgressView(value: data.progress)
                Text("\(String(format: "%.0f", data.progress * 100))%")
            }
        }
    }
}

#Preview {
    DownloadItem(
        data: Download(id: "24389729skjl", title: "Big Buck Bunny", progress: 0.6, is_paused: false)
    )
}
