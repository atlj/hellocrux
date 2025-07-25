import SwiftUI

struct SettingsScreen: View {
    @EnvironmentObject var core: Core

    var body: some View {
        @State var address = ""
        Form {
            Section("Server") {
                HStack {
                    Label("server://", systemImage: "xserve")
                    TextField(
                        text: $address,
                        prompt: Text("192.168.1.127:3000")
                    ) {
                        Text("Server Address")
                    }
                    .font(.body.monospaced())
                    .autocorrectionDisabled()
                    .textContentType(.URL)
                    .textInputAutocapitalization(.never)
                    .onSubmit {}
                }
                Button {} label: {
                    Label("Try Connecting", systemImage: "phone.fill.connection")
                }
            }
        }
        .navigationTitle("Settings")
        .onAppear {
            core.update(.screenChanged(.settings))
        }
    }
}

#Preview {
    SettingsScreen()
        .environmentObject(Core())
}
