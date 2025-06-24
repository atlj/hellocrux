import SwiftUI
import SharedTypes

struct NavigationContainer: View {
    @EnvironmentObject var core: Core
    @State private var screens: [Screen] = []
    
    var body: some View {
        if #available(iOS 16.0, *) {
            NavigationStack(path: $screens) {
                HStack {}
                    .navigationDestination(for: Screen.self) { screen in
                        switch screen {
                            default: ServerAddressEntryScreen()
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
