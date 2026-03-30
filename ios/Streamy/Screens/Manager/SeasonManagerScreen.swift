import SharedTypes
import SwiftUI

struct SeasonManagerScreen: View {
    var media: Media
    var season: UInt32
    var episodes: [UInt32: MediaPaths]
    @EnvironmentObject var core: Core

    var body: some View {
        List {
            Section("Subtitles") {
                Button {
                    core.update(.subtitle(.select(media_id: media.id, season: season)))
                } label: {
                    Label("Download Subtitles", systemImage: "square.and.arrow.down.fill")
                }
            }
        }
        .navigationTitle("\(media.metadata.title) S\(season)")
    }
}

#Preview {
    SeasonManagerScreen(media: PreviewData.idiocracyMedia, season: 2, episodes: [1: .init(media: "", track_name: "", subtitles: [])])
}
