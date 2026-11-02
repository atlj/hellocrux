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
    static var artworkFetchTask: URLSessionTask?

    private var player: AVPlayer {
        if let sharedPlayer = Player.sharedPlayer, Player.sharedPlayerUrl == url {
            return sharedPlayer
        }

        let player = AVPlayer()

        Player.sharedPlayerUrl = url
        Player.sharedPlayer = player

        let videoAsset = AVURLAsset(url: url)
        let playerItem = AVPlayerItem(asset: videoAsset)

        let titleMeta = AVMutableMetadataItem()
        titleMeta.identifier = .commonIdentifierTitle
        titleMeta.value = data.metadata.title as NSString
        playerItem.externalMetadata.append(titleMeta)

        if let episode_id = data.current_episode {
            let subtitleMeta = AVMutableMetadataItem()
            subtitleMeta.identifier = .iTunesMetadataTrackSubTitle
            subtitleMeta.value = String(describing: episode_id) as NSString
            playerItem.externalMetadata.append(subtitleMeta)
        }

        Self.artworkFetchTask?.cancel()
        let artworkURL = URL(string: data.metadata.thumbnail)!
        Self.artworkFetchTask = URLSession.shared.dataTask(
            with: artworkURL,
        ) {
            [weak player, artworkURL] data, response, error in
            if let error {
                Self.logger.error("Couldn't fetch metadata from \(artworkURL)")
                return
            }

            guard let player, let data, let response else { return }

            let mime = (response as? HTTPURLResponse)?.mimeType ?? ""
            let dataType = mime.contains("png")
                ? kCMMetadataBaseDataType_PNG
                : kCMMetadataBaseDataType_JPEG

            let artworkMeta = AVMutableMetadataItem()
            artworkMeta.identifier = .commonIdentifierArtwork
            artworkMeta.value = data as NSData
            artworkMeta.dataType = dataType as String

            player.currentItem?.externalMetadata.append(artworkMeta)
        }
        Self.artworkFetchTask?.resume()

        player.replaceCurrentItem(with: playerItem)

        player.seek(
            to: .init(
                seconds: Double(data.position.getInitialSeconds()),
                preferredTimescale: CMTimeScale(NSEC_PER_SEC),
            ),
        )

        Player.timeObserver = player.addPeriodicTimeObserver(
            forInterval: .init(seconds: 1, preferredTimescale: CMTimeScale(NSEC_PER_SEC)),
            queue: .main,
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
                case .movie(let id, position_seconds: _):
                    .movie(id: id, position_seconds: UInt64(time.seconds))
                case .seriesEpisode(let id, episode_identifier: let episodeID, position_seconds: _):
                    .seriesEpisode(
                        id: id, episode_identifier: episodeID,
                        position_seconds: UInt64(time.seconds),
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
                    .play(.fromCertainEpisode(id: data.position.getId(), episode: nextEpisode)),
                )
            }
        }
        uiViewController.showNextButton(data.next_episode != nil)

        if url != Player.sharedPlayerUrl {
            Self.artworkFetchTask?.cancel()
            uiViewController.player?.dismantle()
        }

        uiViewController.player = player
    }

    static func dismantleUIViewController(
        _ uiViewController: PlayerViewController, coordinator _: (),
    ) {
        artworkFetchTask?.cancel()
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
            id: "id", position: .movie(id: "1", position_seconds: 0),
            media_paths: .init(media: "", track_name: "", subtitles: []),
            metadata: .init(thumbnail: "", title: "Breaking Bad", tmdb_id: nil),
            current_episode: .init(season_no: 1, episode_no: 1),
            next_episode: EpisodeIdentifier(season_no: 1, episode_no: 2),
        ),
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
                    greaterThanOrEqualTo: overlay.bottomAnchor, constant: -24,
                ),
                button.trailingAnchor.constraint(equalTo: overlay.trailingAnchor, constant: -24),
                button.bottomAnchor.constraint(
                    greaterThanOrEqualTo: overlay.safeAreaLayoutGuide.bottomAnchor, constant: -12,
                ),
                button.trailingAnchor.constraint(
                    greaterThanOrEqualTo: overlay.safeAreaLayoutGuide.trailingAnchor, constant: -12,
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
