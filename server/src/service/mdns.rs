use log::info;

/// A service that registers the server using Multicast DNS
pub fn spawn(port: u16) -> mdns_sd::Result<mdns_sd::ServiceDaemon> {
    let daemon = mdns_sd::ServiceDaemon::new()?;

    let service_info = mdns_sd::ServiceInfo::new(
        "_streamy._tcp.local.",
        // TODO set the instance name based on config
        "streamy",
        // TODO set the hostname automatically
        "0.0.0.0.local.",
        "0.0.0.0",
        port,
        None,
    )?
    .enable_addr_auto();

    daemon.register(service_info)?;

    info!("Registered mDNS service");

    Ok(daemon)
}
