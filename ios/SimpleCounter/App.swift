import SwiftUI

@main
struct SimpleCounterApp: App {
  var body: some Scene {
    WindowGroup {
        NavigationContainer()
            .environmentObject(Core())
    }
  }
}
