import SharedTypes
import SwiftUI

struct MediaDetailScreen: View {
    let media: Media
    var body: some View {
        VStack {
            Spacer()
            Button {} label: {
                Label("Play From Beginning", systemImage: "play.fill")
                    .frame(maxWidth: .infinity)
            }
            .buttonStyle(.borderedProminent)
            .padding()
        }
        .background {
            AsyncImage(url: URL(string: media.thumbnail)) { image in
                image.image?
                    .resizable(resizingMode: .stretch)
                    .aspectRatio(contentMode: .fill)
            }
            .ignoresSafeArea()
            .overlay {
                VStack {
                    Rectangle()
                        .frame(maxWidth: .infinity, maxHeight: 200)
                        .foregroundStyle(.linearGradient(.init(colors: [.black, .black.opacity(0)]), startPoint: .top, endPoint: .bottom))
                    Spacer()
                    Rectangle()
                        .frame(maxWidth: .infinity, maxHeight: 200)
                        .foregroundStyle(.linearGradient(.init(colors: [.black, .black.opacity(0)]), startPoint: .bottom, endPoint: .top))
                }
                .ignoresSafeArea()
            }
        }
        .navigationTitle(media.title)
    }
}

#Preview {
    MediaDetailScreen(
        media: Media(id: "1", thumbnail: "https://www.themoviedb.org/t/p/w600_and_h900_bestv2/78lPtwv72eTNqFW9COBYI0dWDJa.jpg", title: "Iron Man")
    )
}
