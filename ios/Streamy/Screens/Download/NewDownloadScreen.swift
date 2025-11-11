import SwiftUI

struct NewDownloadScreen: View {
    @State var hash = ""
    @State var thumbnail = ""
    @State var title = ""

    var disabled: Bool {
        hash.isEmpty || thumbnail.isEmpty || title.isEmpty
    }

    var body: some View {
        Form {
            TextField("Magnet / Torrent File URL", text: $hash)
            TextField("Title", text: $title)
            TextField("Thumbnail Image URL", text: $thumbnail)
            Button {} label: {
                Label("Add New Download", systemImage: "plus")
            }
            .disabled(disabled)
        }
        .navigationTitle("Add New Download")
    }
}

#Preview {
    NewDownloadScreen()
}
