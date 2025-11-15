import SharedTypes
import SwiftUI

struct FileMappingEntry: View {
    @Binding var fileMapping: FileMapping

    var body: some View {
        Text(fileMapping.fileName).monospaced()
        Toggle("Non Media Item", isOn: $fileMapping.isNonMedia)

        if !fileMapping.isNonMedia {
            HStack {
                Selector(title: "Se", value: $fileMapping.seasonNo) {
                    HStack {
                        Text("Season")
                        Spacer()
                    }
                }
            }
            HStack {
                Selector(title: "Ep", value: $fileMapping.episodeNo) {
                    HStack {
                        Text("Episode")
                        Spacer()
                    }
                }
            }
        }
    }
}

struct Selector<T, L>: View where T: BinaryInteger, L: View {
    var title: String
    @Binding var value: T
    @ViewBuilder var Label: L

    var body: some View {
        HStack {
            Label
            TextField(title, value: $value, format: IntegerFormatStyle<T>())
                .textFieldStyle(.roundedBorder)
                .keyboardType(.numberPad)
                .fixedSize()
            Stepper(value: $value, in: 0 ... T(UInt8.max)) {}
                .fixedSize()
        }
    }
}

struct FileMapping {
    var fileName: String
    var seasonNo: UInt
    var episodeNo: UInt
    var isNonMedia = false

    init(fileName: String, seasonNo: UInt, episodeNo: UInt, isNonMedia: Bool = false) {
        self.fileName = fileName
        self.seasonNo = seasonNo
        self.episodeNo = episodeNo
        self.isNonMedia = isNonMedia
    }

    init(fileName: String, episodeIdentifier: EpisodeIdentifier) {
        self.fileName = fileName
        seasonNo = UInt(episodeIdentifier.season_no)
        episodeNo = UInt(episodeIdentifier.episode_no)
    }

    func toMapping() -> (String, EpisodeIdentifier)? {
        if isNonMedia {
            return nil
        }

        return (fileName, EpisodeIdentifier(season_no: UInt32(seasonNo), episode_no: UInt32(episodeNo)))
    }
}

extension FileMapping: Hashable {
    static func == (lhs: Self, rhs: Self) -> Bool {
        lhs.seasonNo == rhs.seasonNo && lhs.episodeNo == rhs.episodeNo && lhs.isNonMedia == rhs.isNonMedia
    }

    func hash(into hasher: inout Hasher) {
        hasher.combine(seasonNo)
        hasher.combine(episodeNo)
        hasher.combine(isNonMedia)
    }
}

@available(iOS 17.0, *)
#Preview {
    @Previewable @State var mapping = FileMapping(fileName: "Season1/hello-world-S1E1.mov", seasonNo: 0, episodeNo: 0)

    Form {
        FileMappingEntry(fileMapping: $mapping)
    }
}
