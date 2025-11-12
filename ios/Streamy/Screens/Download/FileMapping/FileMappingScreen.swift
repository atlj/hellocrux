import SwiftUI

struct FileMappingScreen: View {
    @EnvironmentObject var core: Core
    var id: String
    var overrideLoading: Bool?

    var loading: Bool {
        if let overrideLoading {
            return overrideLoading
        }

        if let existingId = core.view.torrent_contents?.field0 {
            return existingId == id
        }

        return true
    }

    var body: some View {
        List {
            if loading {
                ProgressView("Fetching file list from server.")
            } else {
                HStack {}
            }
        }
    }
}

#Preview {
    FileMappingScreen(id: "", overrideLoading: false)
        .environmentObject(Core())
}
