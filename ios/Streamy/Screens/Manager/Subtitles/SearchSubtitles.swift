import SharedTypes
import SwiftUI

struct SearchSubtitles: View {
    @EnvironmentObject var core: Core
    var media: Media
    var season: UInt32
    @State var language: LanguageCode
    @State var showLanguageSelection = false
    @State var selectedEpisodes = Set<UInt32>()

    var episodes: [UInt32]? {
        switch media.content {
        case .movie:
            nil
        case let .series(episodes):
            Array(episodes[season]!.keys).sorted()
        }
    }

    var nextButtonDisabled: Bool {
        selectedEpisodes.isEmpty
    }

    var body: some View {
        Form {
            Section("Language") {
                Button(Locale.current.localizedString(forLanguageCode: language.iso639_2t())!) {
                    showLanguageSelection = true
                }
            }

            if let episodes {
                Section("Episodes") {
                    ForEach(episodes, id: \.hashValue) { episodeNo in
                        Button {
                            if selectedEpisodes.contains(episodeNo) {
                                selectedEpisodes.remove(episodeNo)
                            } else {
                                selectedEpisodes.insert(episodeNo)
                            }
                        } label: {
                            HStack {
                                Text("\(episodeNo)")
                                Spacer()
                                if selectedEpisodes.contains(episodeNo) {
                                    Image(systemName: "checkmark")
                                        .foregroundStyle(.tint)
                                }
                            }
                        }
                        .foregroundStyle(.primary)
                    }
                }
            }
        }
        .navigationTitle("Search Subtitles for \(media.metadata.title) Season \(season)")
        .sheet(isPresented: $showLanguageSelection) {
            LanguageSelectorSheet(selectedLanguage: $language)
        }
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button {
                    core.update(
                        .subtitle(
                            .search(
                                media_id: media.id,
                                language: language,
                                episodes: .init(season, Array(selectedEpisodes)),
                            ),
                        ),
                    )
                } label: {
                    Label("Search", image: "magnifyingglass")
                }
                .disabled(nextButtonDisabled)
            }
        }
    }
}

#Preview {
    SearchSubtitles(media: PreviewData.idiocracyMedia, season: 1, language: .turkish)
        .environmentObject(Core())
}
