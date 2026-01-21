//
//  DiscoveredAddresses.swift
//  Streamy
//
//  Created by Burak Güner on 22/01/2026.
//

import SwiftUI

struct DiscoveredAddresses: View {
    var addresses: [String]
    @EnvironmentObject var core: Core

    private var _addresses: [Address] {
        addresses.enumerated().map { address in
            Address(id: UInt(address.offset), address: address.element)
        }
    }

    var body: some View {
        // TODO BETTER UX
        VStack {
            Text("Discovered Servers")
                .font(.title3)

            ScrollView {
                ForEach(_addresses) { address in
                    Button {
                        core.update(.serverCommunication(.tryConnecting(address.address)))
                    } label: {
                        Text(address.address)
                    }
                }
                .frame(maxWidth: .infinity)
            }
            .frame(maxWidth: .infinity)
        }
    }
}

private struct Address: Identifiable {
    var id: UInt
    let address: String
}

#Preview {
    DiscoveredAddresses(addresses: [
        "192.168.1.8:3000",
        "192.168.1.2:3000",
        "192.168.1.9:3000",
        "192.168.1.16:3000",
        "192.168.1.2:3000",
    ])
    .environmentObject(Core())
}
