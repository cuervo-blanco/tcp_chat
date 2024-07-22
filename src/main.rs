use mdns::{Error, Record, RecordKind};
use mdns_sd::{ServiceDaemon, ServiceInfo, ServiceEvent};
use std::net::IpAddr;
use async_std::task;
use local_ip_address::local_ip;
use hostname::get;

const SERVICE_NAME: &str = "_tcpchat._tcp.local.";

#[async_std::main]
async fn main() -> Result<(), Error> {
    // Create a new mDNS daemon
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");

    // Obtain local IP address
    let local_ip = local_ip().expect("Failed to obtain local IP address");
    println!("Local IP address: {:?}", local_ip);

    // Get the hostname and format it correctly
    let hostname = get()?;
    let hostname = hostname.to_str().expect("Failed to convert hostname to str");
    let hostname = format!("{}.local.", hostname);
    println!("Hostname: {}", hostname);

    // Define service properties
    let instance_name = "my_instance";
    let port = 5200;
    let properties = [("property_1", "test"), ("property_2", "1234")];

    // Create a new service info
    let my_service = ServiceInfo::new(
        SERVICE_NAME,
        instance_name,
        &hostname,
        local_ip,
        port,
        &properties[..],
    ).unwrap_or_else(|e| {
        eprintln!("Failed to create service info: {}", e);
        std::process::exit(1);
    });

    // Register the service
    mdns.register(my_service).expect("Failed to register our service");

    // Browse for the service
    let receiver = mdns.browse(SERVICE_NAME).expect("Failed to browse");

    // Process service events asynchronously
    task::spawn(async move {
        while let Ok(event) = receiver.recv_async().await {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    println!("ServiceResolved: {:?}", info);
                    println!("Hostname: {:?}", info.get_fullname());
                }
                other_event => {
                    println!("Received other event: {:?}", &other_event);
                }
            }
        }
    });

    // Keep the main thread alive to allow the async task to run
    std::thread::sleep(std::time::Duration::from_secs(60));
    mdns.shutdown().unwrap();

    Ok(())
}

// Helper function to convert mDNS record to IP address
fn _to_ip_addr(record: &Record) -> Option<IpAddr> {
    match record.kind {
        RecordKind::A(addr) => Some(addr.into()),
        RecordKind::AAAA(addr) => Some(addr.into()),
        _ => None,
    }
}

