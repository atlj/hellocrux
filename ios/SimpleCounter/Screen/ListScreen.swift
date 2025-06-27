import SwiftUI

struct ListScreen: View {
    @EnvironmentObject var core: Core
    var body: some View {
        Text(/*@START_MENU_TOKEN@*/"Hello, World!"/*@END_MENU_TOKEN@*/)
    }
}

#Preview {
    ListScreen()
        .environmentObject(Core())
}
