import SharedTypes
import SwiftUI

struct MediaDetailScreen: View {
    @EnvironmentObject var core: Core

    var continueLabel: String {
        var label = "Continue"
        if let playbackDetail = core.view.playback_detail {
            if let lastEpisode = playbackDetail.last_position.episode {
                label = label.appending(" S\(lastEpisode.season) E\(lastEpisode.episode)")
            }

            let formatter = DateComponentsFormatter()
            formatter.zeroFormattingBehavior = .dropLeading
            let formattedTime = formatter.string(from: Double(playbackDetail.last_position.progress_seconds))!
            label = label.appending(" at \(formattedTime)")
        }
        return label
    }

    let media: Media
    var body: some View {
        VStack {
            switch media.content {
            case .movie:
                Spacer()
            case let .series(seriesData):
                Spacer()
                EpisodePicker(id: media.id, series: seriesData)
                    .padding()
                Spacer()
            }
            Button {
                core.update(.play(.fromStart(id: media.id)))
            } label: {
                Label("Play From Beginning", systemImage: "play.fill")
                    .frame(maxWidth: .infinity)
            }
            .buttonStyle(.bordered)
            .padding(.horizontal)
            Button {
                core.update(.play(.fromLastPosition(id: media.id)))
            } label: {
                Label(continueLabel, systemImage: "play.fill")
                    .frame(maxWidth: .infinity)
            }
            .buttonStyle(.borderedProminent)
            .padding()
        }
        .background {
            AsyncImage(url: URL(string: media.metadata.thumbnail)) { image in
                image.image?
                    .resizable(resizingMode: .stretch)
                    .aspectRatio(contentMode: .fill)
            }
            .ignoresSafeArea()
            .overlay {
                VStack {
                    Rectangle()
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                        .foregroundStyle(.ultraThinMaterial)
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
    MediaDetailScreen(
        media: Media(id: "1", metadata: MediaMetaData(thumbnail: "https://m.media-amazon.com/images/M/MV5BMTkzMzM3OTM2Ml5BMl5BanBnXkFtZTgwMDM0NDU3MjI@._V1_FMjpg_UY2048_.jpg", title: "Emoji Movie"), content: MediaContent.movie("test.mp4"))
    )
    .environmentObject(Core())
}

#Preview {
    MediaDetailScreen(
        media: Media(id: "1", metadata: MediaMetaData(thumbnail: "https://m.media-amazon.com/images/M/MV5BMTkzMzM3OTM2Ml5BMl5BanBnXkFtZTgwMDM0NDU3MjI@._V1_FMjpg_UY2048_.jpg", title: "Emoji Movie"), content: MediaContent.series([1: [
            1: "a",
        ]]))
    )
    .environmentObject(Core())
}
