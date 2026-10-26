import SharedTypes
import SwiftUI

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

    var searchResults: [MultiSearchResult]? {
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

    var body: some View {
        ScrollView {
            if let searchResults {
                // TODO: add LazyVGrid
                ForEach(searchResults, id: \.id) { result in
                    Text(result.title)
                }
            }
        }
        .navigationTitle("Search New Media")
        .searchable(text: $searchText, placement: .navigationBarDrawer, prompt: "Search new media")
        .overlay {
            VStack {
                if query.isEmpty {
                    VStack(spacing: 12) {
                        Image(systemName: "magnifyingglass")
                            .font(.largeTitle)
                        Text("Search a title to get started")
                            .font(.title3)
                            .fixedSize()
                    }
                }
            }
        }
        .apply {
            if #available(iOS 18.0, *) {
                $0.searchFocused($searchFocused)
                    .onAppear {
                        searchFocused = true
                    }
            }
        }
        .task(id: query) {
            try? await Task.sleep(for: .seconds(1))

            if Task.isCancelled {
                return
            }

            updateSearch()
        }
    }
}

#Preview {
    NavigationStack {
        NewDownloadSearchScreen()
    }
    .environmentObject(Core.shared)
}
