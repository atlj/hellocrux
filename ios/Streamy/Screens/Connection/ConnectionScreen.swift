import SwiftUI

struct ConnectionScreen: View {
    @State private var displayManualEntry = false

    let addresses = [
        Address(id: 0, label: "Muzpi", address: "192.168.1.12:3000"), Address(id: 1, label: "VPS1", address: "192.168.1.12:3000"),
    ]

//    let addresses = [Address]()

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
                ForEach(addresses) { address in
                    Button {} label: {
                        HStack {
                            Image(systemName: "xserve")
                            VStack(alignment: .leading) {
                                Text(address.label)
                                    .foregroundStyle(.primary)
                                Text(address.address)
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                    .buttonStyle(.plain)
                }
                Button {
                    displayManualEntry.toggle()
                } label: {
                    Label("Connect Manually", systemImage: "pencil")
                }
            } header: {
                HStack {
                    Text("Discovered Servers")
                    if addresses.isEmpty {
                        ProgressView()
                    }
                }
            } footer: {
                Text("If your server wasn't discovered automatically, you can try connecting manually. [See help...](www.test.com)")
            }
            Section {} footer: {}
        }
        .sheet(isPresented: $displayManualEntry) {
            ManualAddressEntryScreen()
        }
    }
}

struct Address: Identifiable {
    var id: UInt8

    var label: String
    var address: String
}

#Preview {
    ConnectionScreen()
}
