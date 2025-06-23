import SharedTypes
import SwiftUI

struct ContentView: View {
  @ObservedObject var core: Core

  var body: some View {
    VStack {
      Image(systemName: "globe")
        .imageScale(.large)
        .foregroundColor(.accentColor)
      HStack {
      }
    }
  }
}

struct ContentView_Previews: PreviewProvider {
  static var previews: some View {
    ContentView(core: Core())
  }
}
