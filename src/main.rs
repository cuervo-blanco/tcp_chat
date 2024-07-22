use mdns::{Error, Record, RecordKind};
use mdns_sd::{ServiceDaemon, ServiceInfo, ServiceEvent};
use std::net::IpAddr;
use async_std;
use async_std::task;
use local_ip_address::local_ip;
use hostname::get;

const SERVICE_NAME: &'static str = "_tcpchat._tcp.local.";

#[async_std::main]
async fn main() -> Result<(), Error> {

    let mdns = ServiceDaemon::new().expect("Failed to create daemon");

    let local_ip = local_ip().unwrap();
    let hostname = get()?;
    let hostname = hostname.to_str().expect("Failed to get hostname");
    let hostname = format!("{}.local.", hostname);
    let hostname = format!("{}.local.", hostname);
    let instance_name = "my_instance";
    let port = 5200;
    let properties = [("property_1", "test"), ("property_2", "1234")];

    let my_service = ServiceInfo::new(
        SERVICE_NAME,
        instance_name,
        &hostname,
        local_ip,
        port,
        &properties[..],
    ).unwrap();

    mdns.register(my_service).expect("Failed to register our service");


    let receiver = mdns.browse(SERVICE_NAME).expect("Failed to browse");

    // Process service events
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

    std::thread::sleep(std::time::Duration::from_secs(60));
    mdns.shutdown().unwrap();

    Ok(())

}

fn _to_ip_addr(record: &Record)-> Option<IpAddr> {
    match record.kind {
        RecordKind::A(addr) => Some(addr.into()),
        RecordKind::AAAA(addr) => Some(addr.into()),
        _ => None,
    }
}
