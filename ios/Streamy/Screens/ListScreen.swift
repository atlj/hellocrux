import SharedTypes
import SwiftUI

struct ListScreen: View {
    @EnvironmentObject var core: Core
    var overrideMediaItems: [Media]?
    let columns = [GridItem(.flexible(), spacing: 16), GridItem(.flexible(), spacing: 16)]

    @State var searchString = ""
    var items: [Media] {
        if let overrideMediaItems {
            return overrideMediaItems
        }

        if let modelItems = core.view.media_items {
            return Array(modelItems.values).sorted {
                $0.metadata.title < $1.metadata.title
            }
        }

        return []
    }

    var filteredItems: [Media] {
        let searchTrimmed = searchString.trimmingCharacters(in: .whitespacesAndNewlines)
        if searchTrimmed.isEmpty {
            return items
        }

        return items.filter { $0.metadata.title.lowercased().contains(searchTrimmed.lowercased()) }
    }

    var body: some View {
        GeometryReader { proxy in
            ScrollView {
                LazyVGrid(columns: columns, spacing: 12) {
                    ForEach(filteredItems, id: \.id) { mediaItem in
                        NavigationLink(value: Screen.detail(mediaItem)) {
                            VStack(alignment: .leading) {
                                AsyncImage(url: URL(string: mediaItem.metadata.thumbnail)) { image in
                                    image
                                        .resizable()
                                } placeholder: {
                                    ProgressView()
                                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                                }
                                .overlay {
                                    VStack {
                                        Spacer()
                                        Text(mediaItem.metadata.title)
                                            .lineLimit(2)
                                            .font(.footnote)
                                            .multilineTextAlignment(.leading)
                                            .foregroundStyle(.white)
                                            .padding()
                                            .padding(.top, 12)
                                            .frame(maxWidth: .infinity, alignment: .leading)
                                            .background(
                                                Rectangle()
                                                    .fill(.linearGradient(colors: [.black.opacity(0), .black.opacity(0.7), .black], startPoint: .top, endPoint: .bottom))
                                            )
                                    }
                                }
                                .frame(height: proxy.size.width * 0.7)
                                .clipShape(RoundedRectangle(cornerRadius: 12.0))
                            }
                        }
                        .foregroundStyle(.primary)
                        .contextMenu {
                            Button("Play From Beginning", systemImage: "play.fill") {
                                core.update(.play(.fromBeginning(id: mediaItem.id)))
                            }
                            Button("Continue", systemImage: "play.fill") {
                                core.update(.play(.fromSavedPosition(id: mediaItem.id)))
                            }
                        }
                    }
                }
                .padding(.horizontal)
            }
            .refreshable {
                // TODO: Fix UI being janky
                core.update(.updateData(.getMedia))
            }
            // TODO: search appears later
            .searchable(text: $searchString, prompt: "Search Media")
            .overlay {
                if items.isEmpty {
                    VStack(spacing: 16.0) {
                        Text("No media items found")
                        Button("Try Again") {
                            core.update(.screenChanged(.list))
                        }
                        .buttonStyle(.borderedProminent)
                    }
                }

                if filteredItems.isEmpty, !searchString.isEmpty {
                    if #available(iOS 17.0, *) {
                        ContentUnavailableView.search
                    }
                }
            }
        }
        .onAppear {
            core.update(.screenChanged(.list))
        }
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                NavigationLink(value: Screen.serverDownloads) {
                    Label("Downloads", systemImage: "square.and.arrow.down")
                }
            }
            ToolbarItem(placement: .topBarTrailing) {
                NavigationLink(value: Screen.settings) {
                    Label("Settings", systemImage: "gearshape")
                }
            }
        }
        .navigationTitle("Media")
    }
}

#Preview {
    NavigationStack {
        ListScreen(
            overrideMediaItems: [
                Media(id: "1", metadata: MediaMetaData(thumbnail: "https://m.media-amazon.com/images/M/MV5BMTkzMzM3OTM2Ml5BMl5BanBnXkFtZTgwMDM0NDU3MjI@._V1_FMjpg_UY2048_.jpg", title: "Emoji Movie"), content: MediaContent.movie(.init(media: "test", subtitles: []))),
            ]
        )
        .environmentObject(Core())
    }
}
