import AVKit
import OSLog
import SharedTypes
import SwiftUI

struct Player: UIViewControllerRepresentable {
    static let logger = Logger()
    var data: ActivePlayerData
    var onProgress: ((UInt64, PlaybackPosition) -> Void)?

    var url: URL {
        URL(string: data.media_paths.media)!
    }

    var subtitles: [Subtitle] {
        data.media_paths.subtitles
    }

    static var sharedPlayer: AVPlayer?
    static var sharedPlayerUrl: URL?
    static var timeObserver: Any?

    private var player: AVPlayer {
        if let sharedPlayer = Player.sharedPlayer, Player.sharedPlayerUrl == url {
            return sharedPlayer
        }

        let player = AVPlayer()

        Player.sharedPlayerUrl = url
        Player.sharedPlayer = player

        let videoAsset = AVURLAsset(url: url)
        let mixComposition = AVMutableComposition()
        let videoTrack = mixComposition.addMutableTrack(
            withMediaType: .video, preferredTrackID: kCMPersistentTrackID_Invalid
        )
        let audioTrack = mixComposition.addMutableTrack(
            withMediaType: .audio, preferredTrackID: kCMPersistentTrackID_Invalid
        )

        // TODO: add a cancellation handler here
        Task { [weak player] in
            guard let duration = try? await videoAsset.load(.duration) else {
                return
            }
            if let videoTrackItem = try await videoAsset.loadTracks(withMediaType: .video).first {
                try videoTrack?.insertTimeRange(
                    CMTimeRangeMake(start: .zero, duration: duration),
                    of: videoTrackItem,
                    at: .zero
                )
            }
            if let audioTrackItem = try await videoAsset.loadTracks(withMediaType: .audio).first {
                try audioTrack?.insertTimeRange(
                    CMTimeRangeMake(start: .zero, duration: duration),
                    of: audioTrackItem,
                    at: .zero
                )
            }

            await withTaskGroup(of: Void.self) { taskGroup in
                for subtitle in subtitles {
                    taskGroup.addTask {
                        let subtitleAsset = AVURLAsset(url: URL(string: subtitle.path)!)
                        guard let loadedSubtitleTrack = try? await subtitleAsset.loadTracks(withMediaType: .subtitle).first else {
                            await Self.logger.warning("Couldn't load subtitle track with path \(subtitle.path)")
                            return
                        }

                        let subtitleTrack = mixComposition.addMutableTrack(withMediaType: .subtitle, preferredTrackID: kCMPersistentTrackID_Invalid)
                        try? subtitleTrack?.insertTimeRange(.init(start: .zero, duration: duration), of: loadedSubtitleTrack, at: .zero)
                        subtitleTrack?.languageCode = subtitle.language_iso639_2t
                    }
                }
            }

            DispatchQueue.main.async { [weak player] in
                guard let player else {
                    return
                }
                player.replaceCurrentItem(with: AVPlayerItem(asset: mixComposition))
            }
        }

        player.seek(
            to: .init(
                seconds: Double(data.position.getInitialSeconds()),
                preferredTimescale: CMTimeScale(NSEC_PER_SEC)
            ))

        Player.timeObserver = player.addPeriodicTimeObserver(
            forInterval: .init(seconds: 1, preferredTimescale: CMTimeScale(NSEC_PER_SEC)),
            queue: .main
        ) { time in
            if time.seconds.rounded(.towardZero) == 0 {
                return
            }

            guard let duration = player.currentItem?.duration,
                  !duration.seconds.isNaN,
                  !duration.seconds.isInfinite
            else {
                return
            }

            let durationSeconds = UInt64(duration.seconds)

            let progress: PlaybackPosition =
                switch data.position {
                case let .movie(id: id, position_seconds: _):
                    .movie(id: id, position_seconds: UInt64(time.seconds))
                case let .seriesEpisode(id: id, episode_identifier: episodeID, position_seconds: _):
                    .seriesEpisode(
                        id: id, episode_identifier: episodeID,
                        position_seconds: UInt64(time.seconds)
                    )
                }

            onProgress?(durationSeconds, progress)
        }

        player.play()

        return player
    }

    func makeUIViewController(context _: Context) -> PlayerViewController {
        let viewController = PlayerViewController()
        viewController.player = player
        viewController.entersFullScreenWhenPlaybackBegins = true
        viewController.allowsPictureInPicturePlayback = true
        viewController.canStartPictureInPictureAutomaticallyFromInline = true
        viewController.updatesNowPlayingInfoCenter = true

        viewController.addNextButton()
        viewController.showNextButton(data.next_episode != nil)

        return viewController
    }

    func updateUIViewController(_ uiViewController: PlayerViewController, context _: Context) {
        uiViewController.onNextButton = {
            if let nextEpisode = data.next_episode {
                Core.shared.update(
                    .play(.fromCertainEpisode(id: data.position.getId(), episode: nextEpisode)))
            }
        }
        uiViewController.showNextButton(data.next_episode != nil)

        if url != Player.sharedPlayerUrl {
            uiViewController.player?.dismantle()
        }

        uiViewController.player = player
    }

    static func dismantleUIViewController(
        _ uiViewController: PlayerViewController, coordinator _: ()
    ) {
        uiViewController.player?.dismantle()
        uiViewController.player = nil
    }

    typealias UIViewControllerType = PlayerViewController
}

extension AVPlayer {
    func dismantle() {
        if let timeObserver = Player.timeObserver {
            removeTimeObserver(timeObserver)
            Player.timeObserver = nil
        }

        replaceCurrentItem(with: nil)

        Player.sharedPlayer = nil
        Player.sharedPlayerUrl = nil
    }
}

#Preview {
    PlayerScreen(
        overrideData: .init(
            position: .movie(id: "1", position_seconds: 0),
            media_paths: .init(media: "", subtitles: []), title: "",
            next_episode: EpisodeIdentifier(season_no: 1, episode_no: 1)
        )
    )
    .environmentObject(Core())
}

class PlayerViewController: AVPlayerViewController {
    var onNextButton: (() -> Void)?

    func addNextButton() {
        if let overlay = contentOverlayView {
            var buttonConfig: UIButton.Configuration = .plain()
            var container = AttributeContainer()
            container.font = .boldSystemFont(ofSize: 40)

            let title = AttributedString("Next Episode", attributes: container)

            buttonConfig.attributedTitle = title
            buttonConfig.buttonSize = .large
            buttonConfig.cornerStyle = .dynamic

            let button = UIButton(configuration: buttonConfig)
            button.backgroundColor = .systemBackground.withAlphaComponent(0.5)
            button.translatesAutoresizingMaskIntoConstraints = false

            overlay.addSubview(button)

            button.isHidden = true
            button.addTarget(self, action: #selector(handleOnNextButton), for: .touchUpInside)

            NSLayoutConstraint.activate([
                button.bottomAnchor.constraint(
                    greaterThanOrEqualTo: overlay.bottomAnchor, constant: -24
                ),
                button.trailingAnchor.constraint(equalTo: overlay.trailingAnchor, constant: -24),
                button.bottomAnchor.constraint(
                    greaterThanOrEqualTo: overlay.safeAreaLayoutGuide.bottomAnchor, constant: -12
                ),
                button.trailingAnchor.constraint(
                    greaterThanOrEqualTo: overlay.safeAreaLayoutGuide.trailingAnchor, constant: -12
                ),
            ])
        }
    }

    func showNextButton(_ show: Bool) {
        contentOverlayView?.subviews.first?.isHidden = !show
    }

    @IBAction func handleOnNextButton() {
        onNextButton?()
    }
}
