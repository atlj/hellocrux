import AVKit
import SwiftUI

struct PlayerScreen: View {
    @State var url: URL
    
    var player: AVPlayer {
        let player = AVPlayer(url: url)
        player.play()
        return player
    }

    var body: some View {
        MyPlayer(player: player)
    }
}

#Preview {
    PlayerScreen(
        url: URL(string:"http://localhost:3000/static/jaho/recording.mov")!
    )
}

struct MyPlayer: UIViewControllerRepresentable {
    var player: AVPlayer
    func makeUIViewController(context: Context) -> AVPlayerViewController {
        let viewController = AVPlayerViewController()
        viewController.player = player
        viewController.entersFullScreenWhenPlaybackBegins = true
        viewController.allowsPictureInPicturePlayback = true
        viewController.canStartPictureInPictureAutomaticallyFromInline = true
        viewController.updatesNowPlayingInfoCenter = true
        return viewController
    }
    
    func updateUIViewController(_ uiViewController: AVPlayerViewController, context: Context) {
        uiViewController.player = player
    }
    
    typealias UIViewControllerType = AVPlayerViewController
}
