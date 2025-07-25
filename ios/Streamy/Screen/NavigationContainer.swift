import SharedTypes
import SwiftUI

struct NavigationContainer: View {
    @EnvironmentObject var core: Core
    @State private var rootView = Screen.startup

    @State private var screens: [Screen] = []

    @ViewBuilder
    func getView(screen: Screen) -> some View {
        switch screen {
        case .serverAddressEntry:
            ServerAddressEntryScreen()
        case .list:
            ListScreen()
        case .startup:
            Spacer()
        case let .detail(media):
            MediaDetailScreen(media: media)
        case .settings:
            SettingsScreen()
        case let .player(id, url, episode, initial_seconds):
            PlayerScreen(url: URL(string: url)!, itemId: id, episode: episode, initialSeconds: initial_seconds)
        }
    }

    var body: some View {
        if #available(iOS 16.0, *) {
            NavigationStack(path: $screens) {
                HStack {
                    getView(screen: rootView)
                }
                .navigationDestination(for: Screen.self) { getView(screen: $0) }
            }.onAppear {
                core.navigationObserver = self
                core.update(.startup)
            }
        } else {
            // Fallback on earlier versions
        }
    }
}

extension NavigationContainer: NavigationObserver {
    func push(screen: SharedTypes.Screen) {
        screens.append(screen)
    }

    func replaceRoot(screen: Screen) {
        rootView = screen
    }
}

#Preview {
    NavigationContainer()
        .environmentObject(Core())
}
