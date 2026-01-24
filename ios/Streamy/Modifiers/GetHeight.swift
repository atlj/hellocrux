import SwiftUI

private struct GetHeightModifier: ViewModifier {
    @Binding var height: CGFloat

    func body(content: Content) -> some View {
        content.background(
            GeometryReader { geo -> Color in
                DispatchQueue.main.async {
                    height = geo.size.height
                }
                return Color.clear
            }
        )
    }
}

extension View {
    func getHeight(height: Binding<CGFloat>) -> some View {
        modifier(GetHeightModifier(height: height))
    }
}
