import SharedTypes
import SwiftUI

struct MediaManagerDetailScreen: View {
    var media: Media

    var body: some View {
        List {
            switch media.content {
            case let .series(seasons):
                Section("Seasons") {
                    ForEach(seasons.sorted { $0.key < $1.key }, id: \.key) { entry in
                        NavigationLink(value: Screen.mediaManagerSeason(media: media, season: entry.key, contents: entry.value, show_download_modal: false)) {
                            Text(String(entry.key))
                        }
                    }
                }
            case let .movie(mediaPaths):
                VStack {}
            }
        }
        .navigationTitle(media.metadata.title)
    }
}

#Preview {
    MediaManagerDetailScreen(
        media: PreviewData.idiocracyMedia,
    )
}
