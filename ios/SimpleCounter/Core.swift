import Foundation
import Serde
import SharedTypes

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
    case let .http(httpOperation):
      switch httpOperation {
      case let .get(urlString):
        let requestId = request.id
        let url = URL(string: urlString)!
        let request = URLRequest(url: url)
        let task = URLSession.shared.dataTask(with: request) { [weak self] data, response, _ in
          DispatchQueue.main.async {
            guard let response = response as? HTTPURLResponse else {
              self?.respond(requestId, response: try! HttpOutput.error.bincodeSerialize())
              return
            }
            let coreResponse = HttpOutput.success(
              data: data == nil ? nil : String(data: data!, encoding: .utf8),
              status_code: Int32(response.statusCode)
            )
            self?.respond(requestId, response: try! coreResponse.bincodeSerialize())
          }
        }
        task.resume()
      }
    }
  }

  private func respond(_ request: Request, response: [UInt8]) {
    respond(request.id, response: response)
  }

  private func respond(_ requestId: UInt32, response: [UInt8]) {
    let requests: [Request] = try! .bincodeDeserialize(input: [UInt8](handleResponse(requestId, Data(response))))
    for request in requests {
      processEffect(request)
    }
  }
}

protocol NavigationObserver {
  func navigate(screen: Screen)
}
