import Combine
import SharedTypes
import SwiftUI

struct ListScreen: View {
    @EnvironmentObject var core: Core
    var overrideMediaItems: [Media]?
    let columns = [GridItem(.flexible(), spacing: 16), GridItem(.flexible(), spacing: 16)]

    @State var searchString = ""
    var items: [Media]? {
        if let overrideMediaItems {
            return overrideMediaItems
        }

        let values: [String: Media].Values? = switch core.view.media_items {
        case let .loading(data):
            data?.values
        case let .success(data):
            data.values
        case .error:
            nil
        }

        guard let values else {
            return nil
        }

        return Array(values).sorted {
            $0.metadata.title < $1.metadata.title
        }
    }

    var error: Bool {
        if case .error = core.view.media_items {
            return true
        }
        return false
    }

    var filteredItems: [Media]? {
        guard let items else {
            return nil
        }

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
                    ForEach(filteredItems ?? [], id: \.id) { mediaItem in
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
                core.update(.updateData(.getMedia))
                var cancellable: AnyCancellable?
                await withCheckedContinuation { continuation in

                    cancellable = core.$view
                        .sink { value in
                            if case .success = value.media_items {
                                continuation.resume()
                            }
                        }
                }
            }
            .searchable(text: $searchString, prompt: "Search Media")
            .overlay {
                switch core.view.media_items {
                case let .loading(data: data):
                    if data == nil {
                        ProgressView()
                    }
                case let .success(data: data):
                    if data.isEmpty {
                        Text("Your media library is empty.")
                    } else {
                        if #available(iOS 17.0, *) {
                            if let filteredItems, filteredItems.isEmpty, !searchString.isEmpty {
                                ContentUnavailableView.search
                            }
                        }
                    }
                case let .error(message: message):
                    VStack(spacing: 16.0) {
                        Text("Couldn't fetch your media library. Reason: \(message)")
                        Button("Try Again") {
                            core.update(.screenChanged(.list))
                        }
                        .buttonStyle(.borderedProminent)
                    }
                }
            }
        }
        .onAppear {
            core.update(.screenChanged(.list))
        }
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                NavigationLink(value: Screen.mediaManager) {
                    Label("Manage Media", systemImage: "server.rack")
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
