import SharedTypes
import SwiftUI

struct DownloadSubtitles: View {
    @EnvironmentObject var core: Core
    var media: Media
    var language: LanguageCode
    var episodes: [EpisodeIdentifier]

    var results: SubtitleSearchResults? {
        guard case let .success(data: successData) = core.view.subtitle_search_results else {
            return nil
        }

        guard successData.mediaId() == media.id, successData.language() == language else {
            return nil
        }

        return successData
    }

    var downloadButtonDisabled: Bool {
        if case .loading = core.view.subtitle_download_results {
            return true
        }
        
        guard let results else {
            return false
        }

        switch results {
        case let .movie(_, _, options):
            return options.isEmpty
        case let .series(_, _, options):
            return options.values.allSatisfy(\.isEmpty)
        }
    }

    /// `S0E0` means this is a movie (wonderful design xd)
    @State private var changedSelections = [EpisodeIdentifier: Int]()

    @State private var sheetData: ([SubtitleSearchResult], Binding<Int>)?

    var body: some View {
        Form {
            Section("Language") {
                Text(Locale.current.localizedString(forLanguageCode: language.iso639_2t())!)
            }

            Section {
                if let results {
                    switch results {
                    case let .movie(_, _, options):
                        let selectionIndex = changedSelections[.init(season_no: 0, episode_no: 0)] ?? 0
                        Button {
                            sheetData = (
                                options,
                                Binding(
                                    get: { selectionIndex },
                                    set: {
                                        changedSelections[.init(season_no: 0, episode_no: 0)] = $0
                                    },
                                ),
                            )
                        } label: {
                            SubtitleOption(result: options.get(selectionIndex))
                        }
                        .disabled(options.count < 2)
                    case let .series(_, _, options):
                        ForEach(options.sorted { $0.key.episode_no > $1.key.episode_no }, id: \.key) { entry in
                            let selectionIndex = changedSelections[entry.key] ?? 0

                            Button {
                                sheetData = (
                                    entry.value,
                                    Binding(
                                        get: { selectionIndex },
                                        set: {
                                            changedSelections[entry.key] = $0
                                        },
                                    ),
                                )
                            } label: {
                                SubtitleOption(episode: entry.key.episode_no, result: entry.value.get(selectionIndex))
                            }
                            .disabled(entry.value.count < 2)
                        }
                    }
                }
            } header: {
                HStack {
                    Text("Available Downloads")
                    if results == nil {
                        ProgressView()
                    }
                }
            } footer: {}
        }
        .navigationTitle("Confirm Subtitle Downloads")
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button {
                    guard let results else {
                        return
                    }
                    let selections: [SubtitleSelection] = {
                        switch results {
                        case let .movie(_, _, options):
                            let selectedIndex = changedSelections[.init(season_no: 0, episode_no: 0)] ?? 0
                            let selection = options.get(selectedIndex)
                            if let selection {
                                return [
                                    .movie(subtitle_id: selection.id),
                                ]
                            } else {
                                return []
                            }
                        case let .series(_, _, options):
                            return options.sorted { $0.key.episode_no > $1.key.episode_no }.compactMap { entry in
                                let selectedIndex = changedSelections[entry.key] ?? 0
                                let selection = entry.value.get(selectedIndex)
                                guard let selection else {
                                    return nil
                                }

                                return .series(subtitle_id: selection.id, episode_identifier: entry.key)
                            }
                        }
                    }()

                    core.update(.subtitle(.download(form: .init(media_id: media.id, language_code: language, selections: selections))))
                }
                label: {
                    HStack {
                        Label("Download", systemImage: "square.and.arrow.down.fill")
                        if case .loading = core.view.subtitle_download_results {
                            ProgressView()
                        }
                    }
                }
                .disabled(downloadButtonDisabled)
            }
        }
        .sheet(
            isPresented: Binding(
                get: { sheetData != nil },
                set: { val in if !val {
                    sheetData = nil
                }},
            ),
        ) {
            if let sheetData {
                SubtitleSelectorSheet(
                    options: sheetData.0,
                    index: sheetData.1,
                )
            }
        }
        .onAppear {
            core.update(
                .subtitle(
                    .fetchSearchResults(media_id: media.id, language: language, episodes: episodes.isEmpty ? nil : episodes),
                ),
            )
        }
    }
}

#Preview {
    DownloadSubtitles(media: PreviewData.idiocracyMedia, language: .turkish, episodes: [.init(season_no: 1, episode_no: 1)])
        .environmentObject(Core())
}

extension SubtitleSearchResults {
    func mediaId() -> String {
        switch self {
        case let .movie(media_id, language, options):
            media_id
        case let .series(media_id, language, options):
            media_id
        }
    }

    func language() -> LanguageCode {
        switch self {
        case let .movie(media_id, language, options):
            language
        case let .series(media_id, language, options):
            language
        }
    }
}
