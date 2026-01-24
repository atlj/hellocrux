import Foundation
import Serde
import SharedTypes

@MainActor
class Core: ObservableObject {
    static let shared = Core()
    var serviceDiscovery: ServiceDiscovery?

    @Published var view: ViewModel
    var navigationObserver: (any NavigationObserver)?

    init() {
        view = try! .bincodeDeserialize(input: [UInt8](Streamy.view()))
    }

    func update(_ event: Event) {
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
            view = try! .bincodeDeserialize(input: [UInt8](Streamy.view()))

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
            case let .remove(key):
                UserDefaults.standard.removeObject(forKey: key)
                respond(request, response: [0])
            }

        case let .navigate(navigationOperation):
            switch navigationOperation {
            case let .push(screen):
                navigationObserver?.push(screen: screen)
                respond(request, response: [])
            case let .replaceRoot(screen):
                navigationObserver?.replaceRoot(screen: screen)
                respond(request, response: [])
            case let .reset(screen):
                navigationObserver?.reset(screen: screen)
                respond(request, response: [])
            }

        case let .http(httpOperation):
            switch httpOperation {
            case let .get(urlString):
                let requestId = request.id
                let url = URL(string: urlString)!
                var request = URLRequest(url: url)
                request.httpMethod = "GET"
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
            case let .post(url: urlString, body: body):
                let requestId = request.id
                let url = URL(string: urlString)!
                var request = URLRequest(url: url)
                request.httpMethod = "POST"
                request.httpBody = Data(body.utf8)
                request.addValue("application/json", forHTTPHeaderField: "Content-Type")
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

        case let .serviceDiscovery(serviceDiscoveryOperation):
            switch serviceDiscoveryOperation {
            case .start:
                serviceDiscovery = ServiceDiscovery()
                serviceDiscovery?.delegate = self
                serviceDiscovery?.start()
                respond(request, response: [])
            case .stop:
                serviceDiscovery?.cancel()
                serviceDiscovery?.delegate = nil
                serviceDiscovery = nil
                respond(request, response: [])
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

extension Core: @preconcurrency ServiceDiscoveryDelegate {
    func discovered(addresses: [DiscoveredService]) {
        DispatchQueue.main.async {
            self.update(.serverCommunication(.discovered(addresses)))
        }
    }
}

protocol NavigationObserver {
    func pop()
    func push(screen: Screen)
    func replaceRoot(screen: Screen)
    func reset(screen: Screen?)
}
