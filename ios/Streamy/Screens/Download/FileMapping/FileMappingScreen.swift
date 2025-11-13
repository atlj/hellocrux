import SharedTypes
import SwiftUI

struct FileMappingScreen: View {
    @EnvironmentObject var core: Core
    var id: String
    var overrideLoading: Bool?

    @State var files: [FileMapping] = []

    var loading: Bool {
        if let overrideLoading {
            return overrideLoading
        }

        if let existingId = core.view.torrent_contents?.field0 {
            return existingId != id
        }

        return true
    }

    var body: some View {
        Form {
            if loading {
                // TODO: center me
                ProgressView("Fetching file list from server.")
            } else {
                ForEach($files, id: \.fileName) { $fileMapping in
                    Section {
                        // TODO: make animations work
                        FileMappingEntry(fileMapping: $fileMapping)
                    }
                }
            }
        }
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                Button("Save") {}
            }
        }
        .navigationTitle("File Mapping")
        .onAppear {
            core.update(.screenChanged(.serverFileMapping(id)))
        }
        .onAppear {
            if loading == false, files.isEmpty {
                setFilesToViewModelState()
            }
        }
        .onChange(of: loading) { value in
            if value == false, files.isEmpty {
                setFilesToViewModelState()
            }
        }
    }

    private func setFilesToViewModelState() {
        // TODO: remove !
        files = core.view.torrent_contents!.field1.map { FileMapping(fileName: $0, episodeIdentifier: $1) }
    }
}

#Preview {
    FileMappingScreen(id: "", overrideLoading: false, files: [
        FileMapping(fileName: "Season1/power-raising-S1E10.mp4", seasonNo: 0, episodeNo: 0),
        FileMapping(fileName: "Season1/suuuuuper-long-title-that-isreaaaallllllyyyyylongS1E20.mp4", seasonNo: 0, episodeNo: 0),

    ])
    .environmentObject(Core())
}
