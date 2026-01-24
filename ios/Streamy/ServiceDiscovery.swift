import Network
import SharedTypes

class ServiceDiscovery {
    let browser = NWBrowser(for: .bonjour(type: "_streamy._tcp", domain: nil), using: .tcp)
    weak var delegate: ServiceDiscoveryDelegate?

    init() {
        browser.browseResultsChangedHandler = { [weak self] results, changes in
            self?.handleBrowseResultsChanged(results: results, changes: changes)
        }
    }

    func start() {
        browser.start(queue: .global(qos: .background))
    }

    func cancel() {
        browser.cancel()
    }

    private func handleBrowseResultsChanged(results: Set<NWBrowser.Result>, changes _: Set<NWBrowser.Result.Change>) {
        // TODO: resolve using changes instead of results to avoid reconnections
        Task {
            let discoveredServices = await withTaskGroup { taskGroup in
                for result in results {
                    taskGroup.addTask {
                        await self.resolve(endpoint: result.endpoint)
                    }
                }

                var discoveredServices = [DiscoveredService]()
                for await connection in taskGroup {
                    guard let connection else {
                        continue
                    }

                    discoveredServices.append(connection)
                }

                return discoveredServices
            }

            delegate?.discovered(addresses: discoveredServices)
        }
    }

    private func resolve(endpoint: NWEndpoint) async -> DiscoveredService? {
        let connection = NWConnection(to: endpoint, using: .tcp)

        let name = if case let .service(name: name, type: _, domain: _, interface: _) = endpoint { name } else { "Unknown" }

        return await withCheckedContinuation { continuation in
            connection.stateUpdateHandler = { state in
                switch state {
                case .ready:
                    break
                case .cancelled, .failed:
                    continuation.resume(returning: nil)
                    return
                default:
                    return
                }

                guard case let .hostPort(host, port) = connection.currentPath?.remoteEndpoint else {
                    continuation.resume(returning: nil)
                    return
                }

                let ip = self.truncNetworkInterface(from: "\(host)")
                let address = "\(ip):\(port)"
                let discoveredService = DiscoveredService(name: name, address: address)

                connection.stateUpdateHandler = nil
                connection.cancel()

                continuation.resume(returning: discoveredService)
            }

            connection.start(queue: .global(qos: .background))
        }
    }

    private func truncNetworkInterface(from ip: String) -> String {
        ip.components(separatedBy: "%").first ?? ip
    }
}

protocol ServiceDiscoveryDelegate: AnyObject {
    func discovered(addresses: [DiscoveredService])
}
