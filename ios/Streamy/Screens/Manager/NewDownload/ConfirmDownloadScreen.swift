import SharedTypes
import SwiftUI

struct DownloadConfirmScreen: View {
    @EnvironmentObject private var core: Core

    var isSeries: Bool
    var tmdbID: UInt32?
    var thumbnail: String
    @State var title: String

    @State private var downloadLink = ""

    var confirmDisabled: Bool {
        if title.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ||
            thumbnail.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ||
            downloadLink.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
        {
            return true
        }

        if URL(string: downloadLink.trimmingCharacters(in: .whitespacesAndNewlines)) == nil {
            return true
        }

        return false
    }

    var body: some View {
        Form {
            Section("Metadata") {
                HStack {
                    LabeledContent {
                        RemoteImage(url: URL(string: thumbnail)!)
                            .frame(height: 200)
                            .accessibilityHidden(true)
                    } label: {
                        Text("Thumbnail")
                            .foregroundStyle(.secondary)
                    }
                    Spacer()
                }
                LabeledContent {
                    Text(isSeries ? "TV Series" : "Movie")
                        .padding(4)
                        .background(RoundedRectangle(cornerRadius: 8).stroke(.foreground))
                } label: {
                    Text("Type")
                        .foregroundStyle(.secondary)
                }
                LabeledContent {
                    TextField("Big Buck Bunny", text: $title)
                        .multilineTextAlignment(.trailing)
                } label: {
                    Text("Title")
                        .foregroundStyle(.secondary)
                }
            }

            Section("Assets") {
                LabeledContent {
                    TextField("https://...", text: $downloadLink)
                        .keyboardType(.URL)
                        .multilineTextAlignment(.trailing)
                } label: {
                    Text("Download URL/Magnet")
                        .foregroundStyle(.secondary)
                }
            }
        }
        .navigationTitle("Confirm Download")
        .toolbar {
            ToolbarItem(placement: .confirmationAction) {
                Button {
                    core.update(.updateData(.addDownload(.init(hash: downloadLink.trimmingCharacters(in: .whitespacesAndNewlines), metadata: .init(thumbnail: thumbnail, title: title.trimmingCharacters(in: .whitespacesAndNewlines), tmdb_id: tmdbID), is_series: isSeries))))
                    core.navigationObserver?.pop(count: 2)
                } label: {
                    Label("Download", systemImage: "square.and.arrow.down.fill")
                        .labelStyle(.titleAndIcon)
                }
                .disabled(confirmDisabled)
            }
        }
    }
}

struct RemoteImage: View {
    let url: URL?
    var aspectRatio: CGFloat = 2 / 3
    var contentMode: ContentMode = .fill
    var cornerRadius: CGFloat = 8

    var body: some View {
        Color.clear
            .aspectRatio(aspectRatio, contentMode: .fit)
            .overlay {
                AsyncImage(url: url, transaction: .init(animation: .easeOut(duration: 0.2))) { phase in
                    switch phase {
                    case let .success(image):
                        image
                            .resizable()
                            .aspectRatio(contentMode: contentMode)
                    case .failure:
                        Image(systemName: "photo")
                            .foregroundStyle(.secondary)
                    case .empty:
                        ProgressView()
                    @unknown default:
                        EmptyView()
                    }
                }
            }
            .background(.quaternary)
            .clipShape(.rect(cornerRadius: cornerRadius))
            .contentShape(.rect(cornerRadius: cornerRadius))
    }
}

#Preview {
    NavigationStack {
        DownloadConfirmScreen(
            isSeries: true, thumbnail: "https://image.tmdb.org/t/p/original/ineLOBPG8AZsluYwnkMpHRyu7L.jpg", title: "Breaking Bad",
        )
    }
}
