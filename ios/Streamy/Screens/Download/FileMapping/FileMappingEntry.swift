import SwiftUI

struct FileMappingEntry: View {
    var fileName: String
    @Binding var season: UInt
    @Binding var episode: UInt
    @Binding var isNonMediaItem: Bool

    @State var str = ""

    var body: some View {
        Text(fileName).monospaced()
            .monospaced()
        Toggle("Non Media Item", isOn: $isNonMediaItem)
        if !isNonMediaItem {
            HStack {
                Selector(title: "Se", value: $episode) {
                    HStack {
                        Text("Season")
                        Spacer()
                    }
                }
            }
            .transition(.scale(scale: 0, anchor: .top))
            HStack {
                Selector(title: "Ep", value: $season) {
                    HStack {
                        Text("Episode")
                        Spacer()
                    }
                }
            }
            .transition(.scale(scale: 0, anchor: .top))
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

@available(iOS 17.0, *)
#Preview {
    @Previewable @State var episode = UInt(9)
    @Previewable @State var season = UInt(10)
    @Previewable @State var nonMediaItem = false

    Form {
        Section {
            FileMappingEntry(fileName: "season1/the-summer-S1E0.mov", season: $season, episode: $episode, isNonMediaItem: $nonMediaItem)
        }
        Section {
            FileMappingEntry(fileName: "season1/the-summer-S1E0.mov", season: $season, episode: $episode, isNonMediaItem: $nonMediaItem)
        }
    }
}
