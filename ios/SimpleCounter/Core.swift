import Foundation
import SharedTypes
import Serde

@MainActor
class Core: ObservableObject {
  @Published var view: ViewModel

  init() {
    view = try! .bincodeDeserialize(input: [UInt8](SimpleCounter.view()))
  }

  func update(_ event: Event) {
    let effects = [UInt8](processEvent(Data(try! event.bincodeSerialize())))

    let requests: [Request] = try! .bincodeDeserialize(input: effects)
    for request in requests {
      processEffect(request)
    }
  }

  func processEffect(_ request: Request) {
    switch request.effect {
    case .render:
      view = try! .bincodeDeserialize(input: [UInt8](SimpleCounter.view()))
    case let .delay(delayOperation):
      switch delayOperation {
      case let .random(min, max):
          let response = DelayOutput.random(.random(in: min...max))
          respond(request, response: try! response.bincodeSerialize())
      case let .delay(time):
        Task {
          try await Task.sleep(nanoseconds: time * 1_000_000)
            let response = DelayOutput.timeUp
            respond(request, response: try! response.bincodeSerialize())
        }
      }
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
    }
  }

    private func respond(_ request: Request, response: [UInt8]) {
        let requests: [Request] = try! .bincodeDeserialize(input: [UInt8](handleResponse(request.id, Data(response))))
    for request in requests {
      processEffect(request)
    }
  }
}
