import SharedTypes
import SwiftUI

struct ManageMediaItem: View {
    var metadata: MediaMetaData

    var body: some View {
        HStack {
            AsyncImage(url: URL(string: metadata.thumbnail)) { image in
                image
                    .resizable()
                    .scaledToFit()
                    .clipShape(RoundedRectangle(cornerRadius: 4.0))
            } placeholder: {
                ProgressView()
            }
            .frame(height: 60)
            .frame(minWidth: 40)
            VStack(alignment: .leading) {
                Text(metadata.title)
                Text("12.0 GiB")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Spacer()
        }
    }
}

#Preview {
    ManageMediaItem(metadata: .init(thumbnail: "https://www.themoviedb.org/t/p/w1280/k75tEyoPbPlfHSKakJBOR5dx1Dp.jpg", title: "Idiocracy"))
}
