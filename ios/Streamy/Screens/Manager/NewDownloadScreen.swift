import SwiftUI

struct NewDownloadScreen: View {
    @EnvironmentObject var core: Core
    @State var hash = ""
    @State var thumbnail = ""
    @State var title = ""
    @State var isSeries = false

    @State var showLoading = false

    var disabled: Bool {
        hash.isEmpty || thumbnail.isEmpty || title.isEmpty
    }

    var body: some View {
        Form {
            TextField("Magnet / Torrent File URL", text: $hash)
            TextField("Title", text: $title)
            TextField("Thumbnail Image URL", text: $thumbnail)
            Toggle(isOn: $isSeries) {
                Text("Series")
            }
            Button {
                Task {
                    core.update(.updateData(.addDownload(.init(hash: hash, metadata: .init(thumbnail: thumbnail, title: title), is_series: isSeries))))
                    showLoading = true
                    // TODO: remove me
                    try? await Task.sleep(for: .seconds(5))
                    showLoading = false
                    core.navigationObserver?.pop()
                }
            } label: {
                Label("Start Downloading", systemImage: "square.and.arrow.down")
            }
            .foregroundStyle(.primary)
            .disabled(disabled)
        }
        .navigationTitle("Download Media")
        .overlay {
            if showLoading {
                VStack {
                    ProgressView()
                }
                .frame(minWidth: 0, maxWidth: .infinity, minHeight: 0, maxHeight: .infinity)
                .background(.ultraThinMaterial.opacity(0.8), ignoresSafeAreaEdges: .all)
            }
        }
    }
}

#Preview {
    NewDownloadScreen()
}
