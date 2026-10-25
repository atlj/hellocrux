import Foundation
import SharedTypes
import SwiftUI

struct TrackSelectionScreen: View {
    @EnvironmentObject var core: Core
    @Environment(\.dismiss) var dismiss
    var preparingMedia: PreparingMedia

    var videoTracks: [(UInt64, String, Duration?)]? {
        preparingMedia.needed_track_selection?.compactMap { track in
            switch track {
            case let .video(id, codec, duration):
                (id, codec, duration)
            default:
                nil
            }
        }
    }

    var audioTracks: [(UInt64, String, Duration?, LanguageCode?)]? {
        preparingMedia.needed_track_selection?.compactMap { track in
            switch track {
            case let .audio(id, codec, duration, language):
                (id, codec, duration, language)
            default:
                nil
            }
        }
    }

    @State var selectedVideoTrackIDState: UInt64?
    @State var selectedAudioTrackIDState: UInt64?

    var selectedVideoTrackID: UInt64? {
        guard let videoTracks else { return nil }

        return videoTracks.count < 2 ? videoTracks.first?.0 : selectedVideoTrackIDState
    }

    var selectedAudioTrackID: UInt64? {
        guard let audioTracks else { return nil }

        return audioTracks.count < 2 ? audioTracks.first?.0 : selectedAudioTrackIDState
    }

    var episodeIdentifier: EpisodeIdentifier? {
        switch preparingMedia.media {
        case .movie:
            nil
        case let .series(_, _, episode):
            episode
        }
    }

    var submitDisabled: Bool {
        if let videoTracks,
           !videoTracks.isEmpty,
           selectedVideoTrackID == nil
        {
            return true
        }

        if let audioTracks,
           !audioTracks.isEmpty,
           selectedAudioTrackID == nil
        {
            return true
        }

        return false
    }

    var body: some View {
        Form {
            if let videoTracks,
               !videoTracks.isEmpty
            {
                Picker(
                    "Video Tracks",
                    selection: .init(
                        get: { selectedVideoTrackID }, set: { selectedVideoTrackIDState = $0 },
                    ),
                ) {
                    ForEach(videoTracks, id: \.0) { id, codec, duration in
                        HStack {
                            Text("Track \(id) ")
                                .bold()
                            Text(codec)
                                .padding(4)
                                .overlay(RoundedRectangle(cornerRadius: 8).stroke(lineWidth: 1))
                                .foregroundStyle(.secondary)
                            if let duration {
                                Spacer()
                                Text(Swift.Duration.seconds(duration.secs).formatted())
                            }
                        }
                        .tag(id)
                    }
                }
                .pickerStyle(.inline)
                .disabled(videoTracks.count < 2)
            }

            if let audioTracks,
               !audioTracks.isEmpty
            {
                Picker(
                    "Audio Tracks",
                    selection: .init(
                        get: { selectedAudioTrackID }, set: { selectedAudioTrackIDState = $0 },
                    ),
                ) {
                    ForEach(audioTracks, id: \.0) { id, codec, duration, language in
                        HStack {
                            if let language {
                                Text(
                                    Locale.current.localizedString(
                                        forLanguageCode: language.iso639_2t(),
                                    )!,
                                )
                                .bold()
                            } else {
                                Text("Unknown Language")
                                    .bold()
                            }

                            Text(codec)
                                .padding(4)
                                .overlay(RoundedRectangle(cornerRadius: 8).stroke(lineWidth: 1))
                                .foregroundStyle(.secondary)

                            if let duration {
                                Spacer()
                                Text(Swift.Duration.seconds(duration.secs).formatted())
                            }
                        }
                        .tag(id)
                    }
                }
                .pickerStyle(.inline)
                .disabled(audioTracks.count < 2)
            }
        }
        .toolbar {
            ToolbarItem(placement: .confirmationAction) {
                Button("Submit Selection") {
                    if submitDisabled {
                        return
                    }

                    guard
                        let selectedTracks = preparingMedia.needed_track_selection?.filter({
                            track in
                            switch track {
                            case let .video(id, _, _):
                                id == selectedVideoTrackID
                            case let .audio(id, _, _, _):
                                id == selectedAudioTrackID
                            case .subtitle:
                                // include all subtitle tracks by default
                                true
                            }
                        })
                    else {
                        // TODO: log
                        return
                    }

                    core.update(
                        .updateData(
                            .submitTrackSelection(
                                .init(media: preparingMedia.media, tracks: selectedTracks),
                            ),
                        ),
                    )

                    dismiss()
                }
                .buttonStyle(.borderedProminent)
                .fixedSize()
                .disabled(submitDisabled)
            }
        }
        .navigationTitle(
            "Select Tracks of \(preparingMedia.metadata.title)\(episodeIdentifier.map { " S\($0.season_no)E\($0.episode_no)" } ?? "")",
        )
        .navigationBarTitleDisplayMode(.inline)
    }
}

#Preview {
    NavigationStack {
        TrackSelectionScreen(
            preparingMedia: .init(
                media: .series(
                    id: "test", path: .init(media: "", track_name: "", subtitles: []),
                    episode: .init(season_no: 3, episode_no: 10),
                ),
                metadata: .init(
                    thumbnail:
                    "https://media.themoviedb.org/t/p/w440_and_h660_face/vZ1RQSDZ3xzBhLqJdrmvpkm2Nbu.jpg",
                    title: "Breaking Bad", tmdb_id: nil,
                ),
                needed_track_selection: [
                    .video(id: 0, codec: "av1", duration: .some(.init(secs: 900, nanos: 90000))),
                    .video(id: 1, codec: "x254", duration: .some(.init(secs: 842_927, nanos: 9))),
                    .video(id: 2, codec: "x254", duration: .some(.init(secs: 938, nanos: 9))),
                    .audio(
                        id: 0, codec: "avi", duration: .some(.init(secs: 898, nanos: 9382)),
                        language: .turkish,
                    ),
                    .audio(
                        id: 1, codec: "acc", duration: .some(.init(secs: 898, nanos: 9382)),
                        language: .english,
                    ),
                    .audio(
                        id: 2, codec: "acc", duration: .some(.init(secs: 898, nanos: 9382)),
                        language: nil,
                    ),
                ],
            ),
        )
    }
}

#Preview("Single Selection") {
    NavigationStack {
        TrackSelectionScreen(
            preparingMedia: .init(
                media: .series(
                    id: "test", path: .init(media: "", track_name: "", subtitles: []),
                    episode: .init(season_no: 3, episode_no: 10),
                ),
                metadata: .init(
                    thumbnail:
                    "https://media.themoviedb.org/t/p/w440_and_h660_face/vZ1RQSDZ3xzBhLqJdrmvpkm2Nbu.jpg",
                    title: "Breaking Bad", tmdb_id: nil,
                ),
                needed_track_selection: [
                    .video(id: 0, codec: "av1", duration: .some(.init(secs: 900, nanos: 90000))),
                    .audio(
                        id: 0, codec: "avi", duration: .some(.init(secs: 898, nanos: 9382)),
                        language: .turkish,
                    ),
                ],
            ),
        )
    }
}

#Preview("Single Video Selection") {
    NavigationStack {
        TrackSelectionScreen(
            preparingMedia: .init(
                media: .series(
                    id: "test", path: .init(media: "", track_name: "", subtitles: []),
                    episode: .init(season_no: 3, episode_no: 10),
                ),
                metadata: .init(
                    thumbnail:
                    "https://media.themoviedb.org/t/p/w440_and_h660_face/vZ1RQSDZ3xzBhLqJdrmvpkm2Nbu.jpg",
                    title: "Breaking Bad", tmdb_id: nil,
                ),
                needed_track_selection: [
                    .video(id: 0, codec: "av1", duration: .some(.init(secs: 900, nanos: 90000))),
                    .audio(
                        id: 0, codec: "avi", duration: .some(.init(secs: 898, nanos: 9382)),
                        language: .turkish,
                    ),
                    .audio(
                        id: 1, codec: "acc", duration: .some(.init(secs: 898, nanos: 9382)),
                        language: .english,
                    ),
                    .audio(
                        id: 2, codec: "acc", duration: .some(.init(secs: 898, nanos: 9382)),
                        language: nil,
                    ),
                ],
            ),
        )
    }
}

#Preview("Single Audio Selection") {
    NavigationStack {
        TrackSelectionScreen(
            preparingMedia: .init(
                media: .series(
                    id: "test", path: .init(media: "", track_name: "", subtitles: []),
                    episode: .init(season_no: 3, episode_no: 10),
                ),
                metadata: .init(
                    thumbnail:
                    "https://media.themoviedb.org/t/p/w440_and_h660_face/vZ1RQSDZ3xzBhLqJdrmvpkm2Nbu.jpg",
                    title: "Breaking Bad", tmdb_id: nil,
                ),
                needed_track_selection: [
                    .video(id: 0, codec: "av1", duration: .some(.init(secs: 900, nanos: 90000))),
                    .video(id: 1, codec: "x254", duration: .some(.init(secs: 842_927, nanos: 9))),
                    .video(id: 2, codec: "x254", duration: .some(.init(secs: 938, nanos: 9))),
                    .audio(
                        id: 0, codec: "avi", duration: .some(.init(secs: 898, nanos: 9382)),
                        language: .turkish,
                    ),
                ],
            ),
        )
    }
}
