import SharedTypes
import SwiftUI

struct ConnectionScreen: View {
    @State private var displayManualEntry = false
    @EnvironmentObject var core: Core

    @State private var lastInteractedService: DiscoveredService?

    var body: some View {
        List {
            Section {
                VStack(alignment: .center) {
                    Text("Streamy")
                        .font(.title.monospaced())
                        .fontWeight(.semibold)
                        .padding(.bottom)
                    Text("Connect to a media server. [Learn more...](www.test.com)")
                }
                .frame(maxWidth: .infinity)
                .padding(.vertical)
            }
            Section {
                ForEach(core.view.discovered_services) { discoveredService in
                    var errored: Bool {
                        lastInteractedService == discoveredService && core.view.connection_state == .error
                    }

                    var loading: Bool {
                        lastInteractedService == discoveredService && core.view.connection_state == .pending
                    }

                    Button {
                        lastInteractedService = discoveredService
                        core.update(.serverCommunication(.tryConnecting(discoveredService.address)))
                    } label: {
                        Label {
                            HStack {
                                VStack(alignment: .leading) {
                                    Text(discoveredService.name)
                                        .foregroundStyle(.primary)
                                    Text(discoveredService.address)
                                        .font(.footnote)
                                        .foregroundStyle(.secondary)
                                }
                                Spacer()
                                if loading {
                                    ProgressView()
                                }
                            }
                        } icon: {
                            Image(systemName: errored ? "exclamationmark.triangle" : "xserve")
                        }
                    }
                    .foregroundStyle(
                        errored ? .red : .primary
                    )
                    .disabled(loading)
                    .alert(
                        "Can't connect to server: \"\(discoveredService.name)\"",
                        isPresented: .constant(errored)
                    ) {
                        Button("Ok") { lastInteractedService = nil }
                    } message: {
                        Text("Make sure you have a valid network connection to \(discoveredService.address) and try again.")
                    }
                }
                Button {
                    displayManualEntry.toggle()
                } label: {
                    Label("Connect Manually", systemImage: "keyboard")
                }
            } header: {
                HStack {
                    Text("Discovered Servers")
                    if core.view.discovered_services.isEmpty {
                        ProgressView()
                    }
                }
            } footer: {
                Text("If your server wasn't discovered automatically, you can try connecting manually. [See help...](www.test.com)")
            }
        }
        .sheet(isPresented: $displayManualEntry) {
            ManualAddressEntryScreen()
        }
        .onAppear {
            core.update(.screenChanged(.serverAddressEntry))
        }
    }
}

extension DiscoveredService: @retroactive Identifiable {
    public var id: String {
        "\(name)\(address)"
    }
}

#Preview {
    ConnectionScreen()
}
