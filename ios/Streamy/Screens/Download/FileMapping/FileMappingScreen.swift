import SharedTypes
import SwiftUI

struct FileMappingScreen: View {
    @EnvironmentObject var core: Core
    var id: String
    var overrideLoading: Bool?

    @State var files: [(String, UInt, UInt, Bool)] = []

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
                ForEach($files, id: \.0) { $file in
                    Section {
                        // TODO: make animations work
                        FileMappingEntry(fileName: file.0, season: $file.1, episode: $file.2, isNonMediaItem: $file.3.animation(.spring))
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
        files = core.view.torrent_contents!.field1.map { ($0, 0, 0, false) }
    }
}

#Preview {
    FileMappingScreen(id: "", overrideLoading: false, files: [("Season1/power-raising-S1E9.mp4", UInt(0), UInt(0), false),
                                                              ("Season1/power-raising-S1E10.mp4", UInt(0), UInt(0), false),
                                                              ("Season1/suuuuuper-long-title-that-isreaaaallllllyyyyylongS1E20.mp4", UInt(0), UInt(0), false),

        ])
        .environmentObject(Core())
}
