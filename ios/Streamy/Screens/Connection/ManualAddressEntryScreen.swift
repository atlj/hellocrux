import SwiftUI

struct ManualAddressEntryScreen: View {
    @State var address = ""
    @FocusState private var focused
    @Environment(\.dismiss) var dismiss
    @EnvironmentObject var core: Core

    @State var lastSubmittedAddress: String?

    private var disableSubmit: Bool {
        core.view.connection_state == .pending
            || address.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    private func submit() {
        if disableSubmit {
            return
        }

        lastSubmittedAddress = address
        core.update(.serverCommunication(.tryConnecting(address)))
    }

    var body: some View {
        NavigationView {
            Form {
                Section {
                    TextField(
                        text: $address,
                        prompt: Text("192.168.1.127:3000")
                    ) {
                        Text("Server Address")
                    }
                    .keyboardType(.URL)
                    .focused($focused, equals: true)
                    .task {
                        focused = true
                    }
                    .font(.body.monospaced())
                    .autocorrectionDisabled()
                    .textContentType(.URL)
                    .textInputAutocapitalization(.never)
                    .submitLabel(.continue)
                    .onSubmit(submit)
                } header: { Text("Server Address") }
            }
            .navigationTitle("Connect Manually")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel", role: .cancel) { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button(action: submit) {
                        HStack {
                            Text("Connect")
                            if core.view.connection_state == .pending {
                                ProgressView()
                            }
                        }
                    }
                    .disabled(disableSubmit)
                }
            }
            .alert(
                "Can't connect",
                isPresented: .constant(core.view.connection_state == .error)
            ) {
                Button("Ok") { lastSubmittedAddress = nil }
            } message: {
                Text("Make sure you have a valid network connection to \(address) and try again.")
            }
        }
    }
}

#Preview {
    ManualAddressEntryScreen()
}
