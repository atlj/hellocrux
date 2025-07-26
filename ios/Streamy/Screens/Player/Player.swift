import AVKit
import SwiftUI

struct Player: UIViewControllerRepresentable {
    var url: URL
    var initialSeconds: UInt64?
    var onProgress: ((CMTime) -> Void)?

    private static var sharedPlayer: AVPlayer?
    private static var sharedPlayerUrl: URL?

    private var player: AVPlayer {
        if let sharedPlayer = Player.sharedPlayer, Player.sharedPlayerUrl == url {
            return sharedPlayer
        }

        let player = AVPlayer(url: url)
        Player.sharedPlayerUrl = url
        Player.sharedPlayer = player

        if let initialSeconds {
            player.seek(to: .init(seconds: Double(initialSeconds), preferredTimescale: CMTimeScale(NSEC_PER_SEC)))
        }

        player.addPeriodicTimeObserver(forInterval: .init(seconds: 1, preferredTimescale: CMTimeScale(NSEC_PER_SEC)), queue: .main) { time in
            if time.seconds.rounded(.towardZero) == 0 {
                return
            }

            onProgress?(time)
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
        uiViewController.player?.replaceCurrentItem(with: nil)
        uiViewController.player = nil
        Player.sharedPlayer = nil
        Player.sharedPlayerUrl = nil
    }

    typealias UIViewControllerType = AVPlayerViewController
}
