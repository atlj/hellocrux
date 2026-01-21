import Network

class ServiceDiscovery {
    let browser = NWBrowser(for: .bonjour(type: "_streamy._tcp", domain: nil), using: .tcp)

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
            let connections = await withTaskGroup { taskGroup in
                for result in results {
                    taskGroup.addTask {
                        await self.resolvePath(to: result.endpoint)
                    }
                }

                var connections = [String]()
                for await connection in taskGroup {
                    guard let connection else {
                        continue
                    }

                    connections.append(connection)
                }

                return connections
            }

            print(connections)
        }
    }

    private func resolvePath(to endpoint: NWEndpoint) async -> String? {
        let connection = NWConnection(to: endpoint, using: .tcp)

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
                let result = "\(ip):\(port)"

                connection.stateUpdateHandler = nil
                connection.cancel()

                continuation.resume(returning: result)
            }

            connection.start(queue: .global(qos: .background))
        }
    }

    private func truncNetworkInterface(from ip: String) -> String {
        ip.components(separatedBy: "%").first ?? ip
    }
}
