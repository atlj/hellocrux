import SharedTypes
import SwiftUI

struct MediaDetailScreen: View {
    @EnvironmentObject var core: Core

    let media: Media
    var body: some View {
        VStack {
            switch media.content {
            case .movie:
                Spacer()
            case let .series(seriesData):
                EpisodePicker(id: media.id, series: seriesData)
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
                Label("Continue", systemImage: "play.fill")
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
                        .frame(maxWidth: .infinity, maxHeight: 300)
                        .foregroundStyle(.linearGradient(.init(colors: [.black, .black.opacity(0.7), .black.opacity(0)]), startPoint: .top, endPoint: .bottom))
                    Spacer()
                    Rectangle()
                        .frame(maxWidth: .infinity, maxHeight: 300)
                        .foregroundStyle(.linearGradient(.init(colors: [.black, .black.opacity(0.7), .black.opacity(0)]), startPoint: .bottom, endPoint: .top))
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
