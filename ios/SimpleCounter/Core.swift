import Foundation
import SharedTypes
import Serde

@MainActor
class Core: ObservableObject {
  @Published var view: ViewModel
  var navigationObserver: (any NavigationObserver)?

  init() {
    view = try! .bincodeDeserialize(input: [UInt8](SimpleCounter.view()))
  }

  func update(_ event: Event) {
    print("event: \(event)")
    let effects = [UInt8](processEvent(Data(try! event.bincodeSerialize())))

    let requests: [Request] = try! .bincodeDeserialize(input: effects)
    for request in requests {
      processEffect(request)
    }
  }

  func processEffect(_ request: Request) {
    print("request: \(request)")
    switch request.effect {
    case .render:
      view = try! .bincodeDeserialize(input: [UInt8](SimpleCounter.view()))
    case let .store(storageOperation):
        switch storageOperation {
        case let .store(key, value):
            UserDefaults.standard.setValue(value, forKey: key)
            respond(request, response: [0])
        case let .get(key):
            let serializer = BincodeSerializer()
            
            let storedValue = UserDefaults.standard.value(forKey: key) as! String?
            
            if let storedValue {
                try! serializer.serialize_option_tag(value: true)
                try! serializer.serialize_str(value: storedValue)
            } else {
                try! serializer.serialize_option_tag(value: false)
            }
            
            let response = serializer.get_bytes()
            respond(request, response: response)
        }
    case let .navigate(navigationOperation):
        switch navigationOperation {
        case let .navigate(screen):
            navigationObserver?.navigate(screen: screen)
        }
    case .serverCommunication(_):
        fatalError()
    }
  }

    private func respond(_ request: Request, response: [UInt8]) {
        let requests: [Request] = try! .bincodeDeserialize(input: [UInt8](handleResponse(request.id, Data(response))))
    for request in requests {
      processEffect(request)
    }
  }
}

protocol NavigationObserver {
    func navigate(screen: Screen)
}
