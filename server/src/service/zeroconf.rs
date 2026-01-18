use std::{any::Any, sync::Arc, time::Duration};

use log::error;
use zeroconf::{MdnsService, ServiceType, prelude::*};

/// A service that registers the server using Multicast DNS
pub fn spawn(port: u16) -> zeroconf::Result<tokio::task::JoinHandle<()>> {
    let service_type = ServiceType::new("http", "tcp").expect("Hardcoded service type to be valid");
    let service_event_loop = {
        let mut service = MdnsService::new(service_type, port);
        service.set_name("Streamy");
        service.set_registered_callback(Box::new(service_registered_callback));
        service.register()?
    };

    Ok(tokio::spawn(async move {
        loop {
            match service_event_loop.poll(Duration::from_secs(1)) {
                Ok(_) => continue,
                Err(err) => {
                    error!("Couldn't poll Zeroconf. Reason: {err}")
                }
            }
        }
    }))
}

fn service_registered_callback(
    result: zeroconf::Result<zeroconf::ServiceRegistration>,
    context: Option<Arc<dyn Any + Send + Sync>>,
) {
    result.inspect_err(|err| error!("Couldn't register Zeroconf. Reason: {err}"));
}
