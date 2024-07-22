use futures_util::{pin_mut, stream::StreamExt};
use mdns::{Error, Record, RecordKind};
use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::{net::IpAddr, time::Duration};
use async_std;
use local_ip_address::local_ip;
use hostname::get;


const SERVICE_NAME: &'static str = "_tcpchat._tcp.local";

#[async_std::main]
async fn main() -> Result<(), Error> {

    let mdns = ServiceDaemon::new().expect("Failed to create daemon");
    
    let instance_name = "my_instance";
    let local_ip = local_ip().unwrap();
    let hostname = get()?;
    let hostname = hostname.to_str().expect("Failed to get hostname");
    let hostname = format!("{}.local.", hostname);
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

    let stream = mdns::discover::all(SERVICE_NAME, Duration::from_secs(15))?
        .listen();
    pin_mut!(stream);

    while let Some(Ok(response)) = stream.next().await {
        let addr = response.records()
            .filter_map(self::to_ip_addr)
            .next();

        if let Some(addr) = addr {
            println!("Found device at {}", addr);
        } else {
            println!("Device does not advertise address");
        }
    }
    Ok(())
}

fn to_ip_addr(record: &Record)-> Option<IpAddr> {
    match record.kind {
        RecordKind::A(addr) => Some(addr.into()),
        RecordKind::AAAA(addr) => Some(addr.into()),
        _ => None,
    }
}
