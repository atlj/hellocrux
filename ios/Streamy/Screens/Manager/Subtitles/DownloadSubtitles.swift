import SharedTypes
import SwiftUI

struct DownloadSubtitles: View {
    @EnvironmentObject var core: Core
    var mediaId: String
    var language: LanguageCode
    var season: UInt32
    var episodes: [UInt32]

    var results: [(UInt32, [SubtitleSearchResult])]? {
        guard case let .success(data: successData) = core.view.subtitle_search_results else {
            return nil
        }

        if successData.season != season || successData.media_id != mediaId || successData.language != language {
            return nil
        }

        return Array(successData.episode_results).sorted { $0.0 > $1.0 }.filter { !$0.1.isEmpty }
    }

    var body: some View {
        Form {
            Section("Language") {
                Text(Locale.current.localizedString(forLanguageCode: language.iso639_2t())!)
            }

            Section("Subtitles") {
                ForEach(results ?? [], id: \.0) { episode, result in
                    // TODO: add subtitle selection feature
                    Button("\(episode): \(result[0].download_count), \(result[0].title)") {}
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
                    let subtitleRequests: [SubtitleRequest] = results!.map { result in
                        SubtitleRequest(episode_identifier: .init(season_no: season, episode_no: result.0), subtitle_id: result.1.first!.id)
                    }

                    core.update(.subtitle(.download(form: .init(media_id: mediaId, requests: subtitleRequests, language_code: language))))
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
            core.update(.updateData(.searchSubtitles(media_id: mediaId, language: language, episodes: .init(season, episodes))))
        }
    }
}

#Preview {
    DownloadSubtitles(mediaId: "Rick_and_Morty", language: .turkish, season: 1, episodes: [1])
        .environmentObject(Core())
}
