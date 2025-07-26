import SwiftUI

struct SettingsScreen: View {
    @EnvironmentObject var core: Core

    var body: some View {
        Form {
            Section("Server") {
                Button("Change Server Address", systemImage: "externaldrive", role: .destructive) {
                    core.update(.serverCommunication(.reset))
                }
            }
        }
        .navigationTitle("Settings")
    }
}

#Preview {
    SettingsScreen()
        .environmentObject(Core())
}
