import SharedTypes
import SwiftUI

struct DownloadSubtitles: View {
    @EnvironmentObject var core: Core
    var media: Media
    var language: LanguageCode
    var episodes: [EpisodeIdentifier]

    var results: [(EpisodeIdentifier, [SubtitleSearchResult])]? {
        guard case let .success(data: successData) = core.view.subtitle_search_results else {
            return nil
        }

        guard successData.media_id == media.id, successData.language == language else {
            return nil
        }

        return successData.episode_results
            .filter { !$0.value.isEmpty }
            .sorted { $0.key.episode_no > $1.key.episode_no }
    }

    var body: some View {
        Form {
            Section("Language") {
                Text(Locale.current.localizedString(forLanguageCode: language.iso639_2t())!)
            }

            Section("Subtitles") {
                ForEach(results ?? [], id: \.0.episode_no) { identifier, result in
                    // TODO: add subtitle selection feature
                    Button("\(identifier.episode_no): \(result[0].download_count), \(result[0].title)") {}
                }
            }
        }
        .overlay {
            if results == nil {
                ProgressView()
            }
        }
        .navigationTitle("Confirm Subtitle Downloads")
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button {
                    let subtitleSelections: [SubtitleSelection] = results!.map { identifier, result in
                        .series(subtitle_id: result.first!.id, episode_identifier: identifier)
                    }

                    core.update(.subtitle(.download(form: .init(media_id: media.id, language_code: language, selections: subtitleSelections))))
                }
                label: {
                    Label("Download", image: "square.and.arrow.down.fill")
                }
            }
        }
        .overlay {
            if case .loading = core.view.subtitle_download_results {
                ProgressView()
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
