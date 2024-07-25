#[allow(unused_imports)]
use std::io::{Write, Read};
use std::sync::{Arc, Mutex};

fn handle_client(mut stream: std::net::TcpStream) {
    let mut message = [0; 960];
    stream.read(&mut message).unwrap();
    let message = std::str::from_utf8(&message).unwrap();
    print!("{}", message);
}

fn main () {
    println!("...preparing to take off.");
    println!("");
    println!("Enter Username:");

    // Take user input (instance name)
    let reader = std::io::stdin();
    let mut instance_name = String::new();
    reader.read_line(&mut instance_name).unwrap();
    let instance_name = instance_name.replace("\n", "").replace(" ", "_");
    
    // Configure Service
    let mdns = mdns_sd::ServiceDaemon::new().expect("Failed to create daemon");
    let service_type = "_tcp_chat._tcp.local.";
    let ip =  local_ip_address::local_ip().unwrap();
    let host_name =  hostname::get()
        .expect("Unable to get host name");
    let host_name = host_name.to_str().expect("Unable to convert to string");
    let host_name = format!("{}.local.", host_name);
    let properties = [("property_1", "attribute_1"), ("property_2", "attribute_2")];

    // Open TCP port 18521
    let port: u16 = 18521;
    let socket_addr = format!("{}:{}", ip, port);
    let listener = std::net::TcpListener::bind(socket_addr)
        .expect("Failed to start listener");

    println!("Starting TCP listener thread...");

    // Read incoming streams from the listener
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                        handle_client(stream)
                },
                Err(_e) => {
                   todo!() 
                }
            }

        };
    });

    // Create Service
    let tcp_chat_service = mdns_sd::ServiceInfo::new(
        service_type,
        &instance_name,
        host_name.as_str(),
        ip,
        port,
        &properties[..],
        ).unwrap();

    // Broadcast service
    mdns.register(tcp_chat_service).expect("Failed to register service");

    // Query for Services
    let receiver = mdns.browse(service_type).expect("Failed to browse");

    // User table to store users discovered and their service information
    let user_table = Arc::new(Mutex::new(std::collections::HashMap::new()));
    let user_table_clone = Arc::clone(&user_table);

    println!("Starting mDNS service thread...");

    // Listen for Services, Respond & Store
    std::thread::spawn(move || {
        while let Ok(event) = receiver.recv() {
            match event {
                mdns_sd::ServiceEvent::ServiceResolved(info) => {
                    // Send request to create tcp connection
                    let addresses = info.get_addresses_v4();
                    for address in addresses {
                        let mut user_table_clone = user_table_clone.lock().unwrap();
                        let user_socket = format!("{}:{}", address, info.get_port());
                        let stream = std::net::TcpStream::connect(user_socket)
                            .expect("Failed to connect to user...");
                        user_table_clone.insert(info.get_fullname().to_string(), stream);
                        let mut username = String::new();
                        for char in info.get_fullname().chars() {
                            if char != '.' {
                                username.push(char); 
                            } else {
                                break;
                            }
                            
                        }
                        
                        println!("{} just connected", username);
                    }
                },
                _ => {

                }
            }
        }
    });

    // Optional: Show Services Discovered
    
    println!("Starting main thread...");
    loop {
        // Take user input 
        let reader = std::io::stdin();
        let mut message: String = String::new();
        reader.read_line(&mut message).unwrap();
        let user_table = user_table.lock().unwrap();
        let message = message.trim();
        // Send message to each socket upon user input Enter
        // Write to the streams in those sockets
        for (user, stream) in user_table.iter() {
            let mut username = String::new();
            for char in user.chars() {
                if char != '.' {
                    username.push(char); 
                } else {
                    break;
                }

            }

            let formatted_message = format!("{}: {}", username, message);
            let mut stream = match stream.try_clone() {
                Ok(stream) => stream,
                Err(e) => {
                    eprintln!("Failed to clone stream: {}", e);
                    continue;
                }
            };
            if let Err(e) = stream.write(formatted_message.as_bytes()){
                eprintln!("Failed to write to stream: {}", e);
            }
            break;
        }
    }
}

