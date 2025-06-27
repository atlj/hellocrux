import SharedTypes
import SwiftUI

struct ListScreen: View {
    @EnvironmentObject var core: Core
    var overrideMediaItems: [Media]?
    let columns = [GridItem(.flexible()), GridItem(.flexible())]

    @State var searchString = ""
    var items: [Media] {
        overrideMediaItems ?? (core.view.media_items ?? [])
    }

    var filteredItems: [Media] {
        let searchTrimmed = searchString.trimmingCharacters(in: .whitespacesAndNewlines)
        if searchTrimmed.isEmpty {
            return items
        }

        return items.filter { $0.title.lowercased().contains(searchTrimmed.lowercased()) }
    }

    var body: some View {
        GeometryReader { proxy in
            ScrollView {
                LazyVGrid(columns: columns) {
                    ForEach(filteredItems, id: \.id) { mediaItem in
                        NavigationLink {
                            MediaDetailScreen(media: mediaItem)
                        } label: {
                            AsyncImage(url: URL(string: mediaItem.thumbnail)) { image in
                                image.resizable()
                            } placeholder: {
                                ProgressView()
                            }
                            .frame(height: proxy.size.width * 0.7)
                            .clipShape(RoundedRectangle(cornerRadius: 12.0))
                        }
                    }
                }
                .padding(.horizontal)
            }
            .searchable(text: $searchString, prompt: "Search Media")
        }
        .onAppear {
            core.update(.screenChanged(.list))
        }
        .toolbar {
            NavigationLink {
                SettingsScreen()
            } label: {
                Label("Settings", systemImage: "gearshape")
            }
        }
        .navigationTitle("Media")
    }
}

#Preview {
    ListScreen(
        overrideMediaItems: [
            Media(id: "1", thumbnail: "https://m.media-amazon.com/images/M/MV5BMTkzMzM3OTM2Ml5BMl5BanBnXkFtZTgwMDM0NDU3MjI@._V1_FMjpg_UY2048_.jpg", title: "Emoji Movie"),
            Media(id: "2", thumbnail: "https://m.media-amazon.com/images/M/MV5BMTkzMzM3OTM2Ml5BMl5BanBnXkFtZTgwMDM0NDU3MjI@._V1_FMjpg_UY2048_.jpg", title: "Emoji Movie"),
            Media(id: "3", thumbnail: "https://m.media-amazon.com/images/M/MV5BMTkzMzM3OTM2Ml5BMl5BanBnXkFtZTgwMDM0NDU3MjI@._V1_FMjpg_UY2048_.jpg", title: "Emoji Movie"),
            Media(id: "4", thumbnail: "https://m.media-amazon.com/images/M/MV5BMTkzMzM3OTM2Ml5BMl5BanBnXkFtZTgwMDM0NDU3MjI@._V1_FMjpg_UY2048_.jpg", title: "Emoji Movie"),
            Media(id: "5", thumbnail: "https://m.media-amazon.com/images/M/MV5BMTkzMzM3OTM2Ml5BMl5BanBnXkFtZTgwMDM0NDU3MjI@._V1_FMjpg_UY2048_.jpg", title: "Emoji Movie"),
        ]
    )
    .environmentObject(Core())
}
