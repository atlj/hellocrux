import SharedTypes
import SwiftUI

struct NavigationContainer: View {
    @EnvironmentObject var core: Core
    @State private var screens: [Screen] = []

    var body: some View {
        if #available(iOS 16.0, *) {
            NavigationStack(path: $screens) {
                HStack {}
                    .navigationDestination(for: Screen.self) { screen in
                        switch screen {
                        case .serverAddressEntry:
                            ServerAddressEntryScreen()
                        case .list:
                            ListScreen()
                        case .startup:
                            ListScreen() // change me
                        case let .detail(media):
                            MediaDetailScreen(media: media)
                        case .settings:
                            SettingsScreen()
                        case let .player(id, url, episode):
                            PlayerScreen(url: URL(string: url)!, itemId: id, episode: episode)
                        }
                    }
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
    func navigate(screen: SharedTypes.Screen) {
        screens.append(screen)
    }
}

#Preview {
    NavigationContainer()
        .environmentObject(Core())
}
