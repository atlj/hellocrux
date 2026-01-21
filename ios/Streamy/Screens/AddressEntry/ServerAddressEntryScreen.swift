import SharedTypes
import SwiftUI

enum Focus: Hashable {
    case onlyField
}

struct ServerAddressEntryScreen: View {
    @EnvironmentObject var core: Core
    @State private var address: String = ""
    @FocusState private var focused: Focus?

    var body: some View {
        VStack {
            Text("Streamy")
                .font(.title.monospaced())
                .fontWeight(.semibold)
                .padding(.bottom, 20)
            if !core.view.discovered_addresses.isEmpty {
                DiscoveredAddresses(addresses: core.view.discovered_addresses)
            }
            VStack {
                HStack {
                    Label("server://", systemImage: core.view.connection_state == .error ? "exclamationmark.triangle" : "xserve")
                        .font(.body.monospaced().bold())
                        .foregroundStyle(core.view.connection_state == .error ? .red : .primary)
                    TextField(
                        text: $address,
                        prompt: Text("192.168.1.127:3000")
                    ) {
                        Text("Server Address")
                    }
                    .focused($focused, equals: .onlyField)
                    .task {
                        focused = .onlyField
                    }
                    .font(.body.monospaced())
                    .autocorrectionDisabled()
                    .textContentType(.URL)
                    .textInputAutocapitalization(.never)
                    .onSubmit {
                        core.update(.serverCommunication(.tryConnecting(address)))
                    }
                }
                Divider()

                Button {
                    core.update(.serverCommunication(.tryConnecting(address)))
                } label: {
                    if core.view.connection_state == .pending {
                        ProgressView()
                    } else {
                        Text("Connect")
                            .frame(maxWidth: .infinity)
                    }
                }
                .buttonStyle(.borderedProminent)
                .disabled(address.isEmpty || core.view.connection_state == .pending)
            }
            .padding()
            .background(.ultraThickMaterial, in: .rect(cornerSize: .init(width: 8.0, height: 8.0)))
            .padding(.horizontal)
        }
        .onAppear {
            core.update(.screenChanged(.serverAddressEntry))
        }

        if core.view.connection_state == .pending {
            ProgressView()
        }
    }
}

#Preview {
    ServerAddressEntryScreen()
        .environmentObject(Core())
}
