import SharedTypes
import SwiftUI

private let columns = [GridItem(.flexible(), spacing: 16), GridItem(.flexible(), spacing: 16)]

struct NewDownloadSearchScreen: View {
    @EnvironmentObject var core: Core

    @State private var searchText = ""
    @FocusState private var searchFocused: Bool

    var query: String {
        searchText.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private func updateSearch() {
        core.update(.updateData(.searchTMDBMedia(query)))
    }

    var mediaLibrarySeries: [MultiSearchResult]? {
        switch core.view.media_items {
        case let .loading(data: data):
            data?.values.compactMap {
                let result = $0.toMultiSearchResult()
                switch result.media_type {
                case .movie:
                    return nil
                case .tv:
                    return result
                }
            }
            .sorted { $0.title < $1.title }
        case let .success(data: data):
            data.values.compactMap {
                let result = $0.toMultiSearchResult()
                switch result.media_type {
                case .movie:
                    return nil
                case .tv:
                    return result
                }
            }
            .sorted { $0.title < $1.title }
        case .error(message: _):
            nil
        }
    }

    var mediaLibrarySearchResults: [MultiSearchResult]? {
        if query.isEmpty {
            return mediaLibrarySeries
        }

        return mediaLibrarySeries?.filter { $0.title.lowercased().contains(query.lowercased()) }
    }

    var tmdbSearchResults: [MultiSearchResult]? {
        if core.view.tmdb_search_results?.field0 != query {
            return nil
        }

        guard let queryState = core.view.tmdb_search_results?.field1 else { return nil }

        switch queryState {
        case let .loading(data: data):
            return data?.results
        case let .success(data: data):
            return data.results
        case .error:
            return nil
        }
    }

    var isTmdbLoading: Bool {
        if query.isEmpty {
            return false
        }

        if core.view.tmdb_search_results?.field0 != query {
            return true
        }

        if case .loading = core.view.tmdb_search_results?.field1 {
            return true
        }

        return false
    }

    var body: some View {
        GeometryReader { proxy in
            ScrollView {
                LazyVGrid(columns: columns, spacing: 12) {
                    if let mediaLibrarySearchResults,
                       !mediaLibrarySearchResults.isEmpty
                    {
                        Section {
                            // TODO: join this style with list screen
                            ForEach(mediaLibrarySearchResults, id: \.title) { result in
                                NavigationLink(value: Screen.list) {
                                    VStack(alignment: .leading) {
                                        AsyncImage(url: URL(string: result.poster_path)) { image in
                                            image
                                                .resizable()
                                        } placeholder: {
                                            ProgressView()
                                                .frame(maxWidth: .infinity, maxHeight: .infinity)
                                        }
                                        .overlay {
                                            VStack {
                                                Spacer()
                                                Text(result.title)
                                                    .lineLimit(2)
                                                    .font(.footnote)
                                                    .multilineTextAlignment(.leading)
                                                    .foregroundStyle(.white)
                                                    .padding()
                                                    .padding(.top, 12)
                                                    .frame(maxWidth: .infinity, alignment: .leading)
                                                    .background(
                                                        Rectangle()
                                                            .fill(.linearGradient(colors: [.black.opacity(0), .black.opacity(0.7), .black], startPoint: .top, endPoint: .bottom)),
                                                    )
                                            }
                                        }
                                        .frame(height: proxy.size.width * 0.7)
                                        .clipShape(RoundedRectangle(cornerRadius: 12.0))
                                    }
                                }
                                .foregroundStyle(.primary)
                            }
                            .padding(.bottom)
                        } header: {
                            HStack {
                                Text("Add Episodes to Existing TV Series")
                                    .font(.callout)
                                    .fontWeight(.medium)
                                    .foregroundStyle(.secondary)
                                Spacer()
                            }
                        }
                    }

                    Section {
                        if let tmdbSearchResults,
                           !tmdbSearchResults.isEmpty
                        {
                            // TODO: join this style with list screen
                            ForEach(tmdbSearchResults, id: \.id) { result in
                                NavigationLink(value: Screen.list) {
                                    VStack(alignment: .leading) {
                                        AsyncImage(url: URL(string: result.poster_path)) { image in
                                            image
                                                .resizable()
                                        } placeholder: {
                                            ProgressView()
                                                .frame(maxWidth: .infinity, maxHeight: .infinity)
                                        }
                                        .overlay {
                                            VStack {
                                                Spacer()
                                                Text(result.title)
                                                    .lineLimit(2)
                                                    .font(.footnote)
                                                    .multilineTextAlignment(.leading)
                                                    .foregroundStyle(.white)
                                                    .padding()
                                                    .padding(.top, 12)
                                                    .frame(maxWidth: .infinity, alignment: .leading)
                                                    .background(
                                                        Rectangle()
                                                            .fill(.linearGradient(colors: [.black.opacity(0), .black.opacity(0.7), .black], startPoint: .top, endPoint: .bottom)),
                                                    )
                                            }
                                        }
                                        .frame(height: proxy.size.width * 0.7)
                                        .clipShape(RoundedRectangle(cornerRadius: 12.0))
                                    }
                                }
                                .foregroundStyle(.primary)
                            }
                        }
                    } header: {
                        VStack {
                            HStack {
                                Text("Add New Media")
                                    .font(.callout)
                                    .fontWeight(.medium)
                                    .foregroundStyle(.secondary)
                                if isTmdbLoading {
                                    ProgressView()
                                }
                                Spacer()
                            }

                            if query.isEmpty {
                                VStack(spacing: 8) {
                                    Image(systemName: "magnifyingglass")
                                        .font(.largeTitle)
                                    Text("Search for a title to add new media")
                                        .font(.headline)
                                }
                                .foregroundStyle(.tertiary)
                                .padding(.vertical, 8)
                            } else {
                                if #available(iOS 17.0, *) {
                                    if let tmdbSearchResults,
                                       tmdbSearchResults.isEmpty,
                                       !isTmdbLoading
                                    {
                                        ContentUnavailableView.search
                                            .padding(.vertical, 8)
                                    }
                                }
                            }
                        }
                    }
                }
                .padding(.horizontal)
            }
            .navigationTitle("Add New Download")
            .searchable(text: $searchText, prompt: "Search new media")
            .apply {
                if #available(iOS 18.0, *) {
                    $0.searchFocused($searchFocused)
                        .onAppear {
                            searchFocused = true
                        }
                }
            }
            .task(id: query) {
                try? await Task.sleep(for: .milliseconds(600))
                if Task.isCancelled {
                    return
                }

                updateSearch()
            }
        }
    }
}

extension Media {
    func toMultiSearchResult() -> MultiSearchResult {
        let type: MediaType = switch content {
        case .movie: .movie
        case .series: .tv
        }

        return .init(id: 0, title: metadata.title, poster_path: metadata.thumbnail, media_type: type)
    }
}

#Preview {
    var core: Core {
        let core = Core.shared
        core.view.media_items = .success(data: ["breaking_bad": .init(id: "breaking_bad", metadata: .init(thumbnail: "https://image.tmdb.org/t/p/original/ineLOBPG8AZsluYwnkMpHRyu7L.jpg", title: "Breaking Bad", tmdb_id: nil), content: .series([:]))])

        return core
    }

    NavigationStack {
        NewDownloadSearchScreen()
    }
    .environmentObject(core)
}
