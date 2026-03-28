import SharedTypes
import SwiftUI

struct SubtitleSelectorSheet: View {
    @Environment(\.dismiss) private var dismiss
    var options: [SubtitleSearchResult]
    @Binding var index: Int

    var body: some View {
        NavigationStack {
            Form {
                ForEach(options.enumerated(), id: \.element.id) { option in
                    Button {
                        index = option.offset
                        dismiss()
                    } label: {
                        HStack {
                            SubtitleOption(result: option.element)
                            Spacer()
                            if index == option.offset {
                                Image(systemName: "checkmark")
                            }
                        }
                    }
                }
            }
            .navigationTitle("Available Downloads")
        }
    }
}

@available(iOS 17.0, *)
#Preview {
    @Previewable @State var index = 0
    SubtitleSelectorSheet(options: [
        .init(id: 123, title: "Big.buck.bunny.HDTV.x264-en", download_count: 6722),
    ], index: $index)
}
