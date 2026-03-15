import SharedTypes
import SwiftUI

struct NavigationContainer: View {
    @EnvironmentObject var core: Core
    @State private var rootView = Screen.startup

    @State private var navPath = NavigationPath()

    var body: some View {
        NavigationStack(path: $navPath) {
            HStack {
                getView(screen: rootView)
            }
            .navigationDestination(for: Screen.self) { getView(screen: $0) }
        }.onAppear {
            core.navigationObserver = self
            core.update(.startup)
        }.onDisappear {
            core.navigationObserver = nil
        }
    }

    @ViewBuilder
    func getView(screen: Screen) -> some View {
        switch screen {
        case .serverAddressEntry:
            ConnectionScreen()
        case .list:
            ListScreen()
        case .startup:
            Spacer()
        case let .detail(media):
            MediaDetailScreen(media: media)
        case .settings:
            SettingsScreen()
        case .player:
            PlayerScreen()
        case .mediaManager:
            MediaManagerScreen()
        case let .mediaManagerDetail(media):
            MediaManagerDetailScreen(media: media)
        case let .mediaManagerSeason(media: media, season: season, contents: episodes, show_download_modal: showDownloadModal):
            SeasonManagerScreen(media: media, season: season, episodes: episodes)
        case .addDownload:
            NewDownloadScreen()
        case let .serverFileMapping(id):
            FileMappingScreen(id: id)
        case let .subtitleSelection(media: media, season: season, pre_selected_episodes: pre_selected_episodes, pre_selected_language: pre_selected_language):
            SearchSubtitles(media: media, season: season, language: pre_selected_language, selectedEpisodes: Set(pre_selected_episodes))
        case .subtitleSearchResult:
            #warning("hey 2")
        }
    }
}

extension NavigationContainer: NavigationObserver {
    func push(screen: SharedTypes.Screen) {
        navPath.append(screen)
    }

    func pop() {
        navPath.removeLast()
    }

    func replaceRoot(screen: Screen) {
        rootView = screen
    }

    func reset(screen: Screen?) {
        if let screen {
            rootView = screen
        }
        navPath.removeLast(navPath.count)
    }
}

#Preview {
    NavigationContainer()
        .environmentObject(Core())
}
