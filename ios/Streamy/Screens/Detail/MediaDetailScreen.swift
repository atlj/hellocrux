import SharedTypes
import SwiftUI

struct MediaDetailScreen: View {
    @EnvironmentObject var core: Core

    var continueLabel: String? {
        guard let playbackDetail = core.view.playback_detail,
              playbackDetail.last_position.progress_seconds != 0
        else {
            return nil
        }

        var label = "Continue"
        if let lastEpisode = playbackDetail.last_position.episode {
            label = label.appending(" S\(lastEpisode.season) E\(lastEpisode.episode)")
        }

        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [
            .hour,
            .minute,
            .second,
        ]
        let formattedTime = formatter.string(from: Double(playbackDetail.last_position.progress_seconds))!
        label = label.appending(" at \(formattedTime)")
        return label
    }

    let media: Media
    var body: some View {
        List {
            if case let .series(seriesData) = media.content {
                EpisodePicker(id: media.id, series: seriesData)
            }
        }
        .listStyle(.plain)
        .toolbar {
            ToolbarItemGroup(placement: .bottomBar) {
                if let continueLabel {
                    Button {
                        core.update(.play(.fromStart(id: media.id)))
                    } label: {
                        Label("Play", systemImage: "play.fill")
                    }
                    .buttonStyle(.bordered)
                    .foregroundStyle(.primary)

                    Button {
                        core.update(.play(.fromLastPosition(id: media.id)))
                    } label: {
                        Label(continueLabel, systemImage: "play.fill")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.borderedProminent)
                } else {
                    Button {
                        core.update(.play(.fromStart(id: media.id)))
                    } label: {
                        Label("Play", systemImage: "play.fill")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.bordered)
                    .foregroundStyle(.primary)
                }
            }
            ToolbarItem(placement: .bottomBar) {}
        }
        .labelStyle(.titleAndIcon)
        .background {
            AsyncImage(url: URL(string: media.metadata.thumbnail)) { image in
                image.image?
                    .resizable(resizingMode: .stretch)
                    .aspectRatio(contentMode: .fill)
            }
            .blur(radius: 18)
            .ignoresSafeArea()
            .overlay {
                VStack {
                    Rectangle()
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                        .foregroundStyle(Color(UIColor.systemBackground).opacity(0.5))
                }
                .ignoresSafeArea()
            }
        }
        .navigationTitle(media.metadata.title)
        .onAppear {
            core.update(.screenChanged(.detail(media)))
        }
    }
}

#Preview {
    if #available(iOS 16.0, *) {
        NavigationStack {
            MediaDetailScreen(
                media: Media(id: "1", metadata: MediaMetaData(thumbnail: "https://m.media-amazon.com/images/M/MV5BMTkzMzM3OTM2Ml5BMl5BanBnXkFtZTgwMDM0NDU3MjI@._V1_FMjpg_UY2048_.jpg", title: "Emoji Movie"), content: MediaContent.movie("test.mp4"))
            )
            .environmentObject(Core())
        }
    } else {
        // Fallback on earlier versions
    }
}

#Preview {
    if #available(iOS 16.0, *) {
        NavigationStack {
            MediaDetailScreen(
                media: Media(id: "1", metadata: MediaMetaData(thumbnail: "https://m.media-amazon.com/images/M/MV5BMTkzMzM3OTM2Ml5BMl5BanBnXkFtZTgwMDM0NDU3MjI@._V1_FMjpg_UY2048_.jpg", title: "Emoji Movie"), content: MediaContent.series([1: [
                    1: "a",
                ], 2: [1: "b", 3: "c"]]))
            )
            .environmentObject(Core())
        }
    } else {
        // Fallback on earlier versions
    }
}
