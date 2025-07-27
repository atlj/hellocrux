import AVKit
import SharedTypes
import SwiftUI

struct Player: UIViewControllerRepresentable {
    var data: ActivePlayerData
    var onProgress: ((UInt64, PlaybackPosition) -> Void)?

    var url: URL {
        URL(string: data.url)!
    }

    private static var sharedPlayer: AVPlayer?
    private static var sharedPlayerUrl: URL?
    private static var timeObserver: Any?

    private var player: AVPlayer {
        if let sharedPlayer = Player.sharedPlayer, Player.sharedPlayerUrl == url {
            return sharedPlayer
        }

        let player = AVPlayer(url: url)
        Player.sharedPlayerUrl = url
        Player.sharedPlayer = player

        player.seek(to: .init(seconds: Double(data.position.getInitialSeconds()), preferredTimescale: CMTimeScale(NSEC_PER_SEC)))

        Player.timeObserver = player.addPeriodicTimeObserver(forInterval: .init(seconds: 1, preferredTimescale: CMTimeScale(NSEC_PER_SEC)), queue: .main) { time in
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

            let progress: PlaybackPosition = switch data.position {
            case let .movie(id: id, position_seconds: _):
                .movie(id: id, position_seconds: UInt64(time.seconds))
            case let .seriesEpisode(id: id, episode_identifier: episodeID, position_seconds: _):
                .seriesEpisode(id: id, episode_identifier: episodeID, position_seconds: UInt64(time.seconds))
            }

            onProgress?(durationSeconds, progress)
        }

        player.play()

        return player
    }

    func makeUIViewController(context _: Context) -> AVPlayerViewController {
        let viewController = AVPlayerViewController()
        viewController.player = player
        viewController.entersFullScreenWhenPlaybackBegins = true
        viewController.allowsPictureInPicturePlayback = true
        viewController.canStartPictureInPictureAutomaticallyFromInline = true
        viewController.updatesNowPlayingInfoCenter = true
        return viewController
    }

    func updateUIViewController(_ uiViewController: AVPlayerViewController, context _: Context) {
        uiViewController.player = player
    }

    static func dismantleUIViewController(_ uiViewController: AVPlayerViewController, coordinator _: ()) {
        if let timeObserver = Player.timeObserver {
            uiViewController.player?.removeTimeObserver(timeObserver)
            Player.timeObserver = nil
        }

        uiViewController.player?.replaceCurrentItem(with: nil)
        uiViewController.player = nil
        Player.sharedPlayer = nil
        Player.sharedPlayerUrl = nil
    }

    typealias UIViewControllerType = AVPlayerViewController
}
