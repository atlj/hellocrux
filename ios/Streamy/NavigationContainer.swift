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
            ServerAddressEntryScreen()
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
        case .serverDownloads:
            DownloadScreen()
        case .addDownload:
            Spacer()
        }
    }
}

extension NavigationContainer: NavigationObserver {
    func push(screen: SharedTypes.Screen) {
        navPath.append(screen)
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
