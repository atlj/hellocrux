import AVKit
import SwiftUI

struct Player: UIViewControllerRepresentable {
    var player: AVPlayer
    var onProgress: (_ seconds: UInt64) -> Void

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
    }

    typealias UIViewControllerType = AVPlayerViewController
}
