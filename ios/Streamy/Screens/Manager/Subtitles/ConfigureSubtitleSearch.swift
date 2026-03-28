import SharedTypes
import SwiftUI

struct ConfigureSubtitleSearch: View {
    @EnvironmentObject var core: Core
    var media: Media

    @State var language: LanguageCode
    @State private var showLanguageSelection = false

    @State var season: UInt32?
    @State var selectedEpisodes: Set<UInt32>

    var episodes: [UInt32]? {
        guard case let .series(contents) = media.content else {
            return nil
        }

        guard let season else {
            return nil
        }

        return contents[season].map { Array($0.keys) }
    }

    var nextButtonDisabled: Bool {
        if season == nil {
            return false
        }

        return selectedEpisodes.isEmpty
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
        .navigationTitle("Search Subtitles for \(media.metadata.title)\(season.map { "Season \($0)" } ?? "")")
        .sheet(isPresented: $showLanguageSelection) {
            LanguageSelectorSheet(selectedLanguage: $language)
        }
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button {
                    core.update(
                        .subtitle(
                            .search(
                                media: media,
                                language: language,
                                episodes: season.map { season in
                                    (season, selectedEpisodes)
                                }
                                .map { season, episodes in
                                    episodes.map {
                                        EpisodeIdentifier(season_no: season, episode_no: $0)
                                    }
                                },
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
    ConfigureSubtitleSearch(media: PreviewData.idiocracyMedia, language: .turkish, season: 1, selectedEpisodes: Set())
        .environmentObject(Core())
}
