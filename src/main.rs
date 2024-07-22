use futures_util::{pin_mut, stream::StreamExt};
use mdns::{Error, Record, RecordKind};
use std::{net::IpAddr, time::Duration};
use async_std;


const SERVICE_NAME: &'static str = "_tcpchat._tcp.local";

#[async_std::main]
async fn main() -> Result<(), Error> {
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
