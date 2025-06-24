import SwiftUI

struct ServerAddressEntryScreen: View {
    @EnvironmentObject var core: Core
    
    var body: some View {
        Text(/*@START_MENU_TOKEN@*/"Hello, World!"/*@END_MENU_TOKEN@*/)
    }
}

#Preview {
    ServerAddressEntryScreen()
        .environmentObject(Core())
}
