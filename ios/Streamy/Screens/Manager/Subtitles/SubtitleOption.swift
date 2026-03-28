import SharedTypes
import SwiftUI

struct SubtitleOption: View {
    var episode: UInt32?
    var result: SubtitleSearchResult?

    var downloadMessage: AttributedString? {
        guard let result else {
            return nil
        }

        let formatter = NumberFormatter()
        formatter.numberStyle = .decimal
        formatter.usesGroupingSeparator = true
        formatter.groupingSeparator = ","

        var downloadString = AttributedString(formatter.string(for: result.download_count)!)
        downloadString.font = .body.bold()

        downloadString.append(AttributedString(" downloads"))
        return downloadString
    }

    var body: some View {
        HStack(spacing: 12) {
            if let episode {
                VStack {
                    Text("Ep.")
                    Text(episode, format: .number)
                        .font(.title)
                        .monospaced()
                }
            }
            VStack(alignment: .leading) {
                if let result, let downloadMessage {
                    Text(result.title)
                        .foregroundStyle(.secondary)
                    Text(downloadMessage)
                } else {
                    Text("No subtitle found")
                }
            }
        }
    }
}

#Preview {
    Form {
        Section("Subtitles") {
            SubtitleOption(result: .init(id: 88, title: "mmxav.Dolby.atmos.new.x264-subs", download_count: 5207))
            SubtitleOption(episode: 11, result: .init(id: 88, title: "mmxav.Dolby.atmos.new.x264-subs", download_count: 5207))
            Button {} label: {
                SubtitleOption(episode: 12, result: .init(id: 88, title: "mmxav.Dolby.atmos.new.x264-subs", download_count: 5207))
            }
            Button {} label: {
                SubtitleOption(episode: 13, result: nil)
            }
            .disabled(true)
            Button {} label: {
                SubtitleOption(result: nil)
            }
            .disabled(true)
        }
    }
}
