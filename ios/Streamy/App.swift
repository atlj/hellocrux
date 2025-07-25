import AVFAudio
import SwiftUI

@main
struct SimpleCounterApp: App {
    init() {
        try? AVAudioSession.sharedInstance().setCategory(.playback, mode: .moviePlayback)
    }

    var body: some Scene {
        WindowGroup {
            NavigationContainer()
                .environmentObject(Core.shared)
        }
    }
}
