#[allow(unused_imports)]
use std::io::{Write, Read};
use std::sync::{Arc, Mutex};

fn handle_client(mut stream: std::net::TcpStream) {
<<<<<<< HEAD
    let mut buffer = [0; 960];
    match stream.read(&mut buffer) {
        Ok(_) => println!("Message read successfully"),
        Err(e) => println!("Failed to read message: {}", e),
    }
    let message = std::str::from_utf8(&buffer).unwrap();
    std::io::stdout
        //Working on this
    println!("{}", message);
=======
    let mut message = [0; 512];
    match stream.read(&mut message) {
        Ok(size) => {
            if size == 0 {
                println!("Connection closed by the client");
                return;
            }
            match std::str::from_utf8(&message[..size]){
                Ok(message) => println!("{}", message),
                Err(e) => println!("Failed to convert message to UTF-8: {}", e),
            }
        }
        Err(e) => println!("Failed to read message: {}", e),
    }
>>>>>>> refs/remotes/origin/main
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
    println!("ServiceDaemon created");
    let service_type = "_tcp_chat._tcp.local.";
    let ip =  local_ip_address::local_ip().unwrap();
    println!("Local IP address: {}", ip);
    let host_name =  hostname::get()
        .expect("Unable to get host name");
    let host_name = host_name.to_str()
        .expect("Unable to convert to string");
    let host_name = format!("{}.local.", host_name);
    println!("Host name: {}", host_name);
    let properties = [("property_1", "attribute_1"), ("property_2", "attribute_2")];

    // Open TCP port 18521 (listen to connections)
    let port: u16 = 18521;
    let socket_addr = format!("{}:{}", ip, port);
    let listener = std::net::TcpListener::bind(socket_addr.clone())
        .expect("Failed to start listener");
    println!("TCP Listener bound to address: {}", socket_addr);

    println!("Starting TCP listener thread...");

    // Read incoming streams from the listener
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New client connected");
                    handle_client(stream)
                },
                Err(e) => {
                    println!("Failed to accept connection: {}", e);
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
    println!("Service Info created");

    // Broadcast service
    mdns.register(tcp_chat_service).expect("Failed to register service");
    println!("Service registered");

    // Query for Services
    let receiver = mdns.browse(service_type).expect("Failed to browse");
    println!("Browsing for services");

    // User table to store users discovered and their service information
    let user_table = Arc::new(Mutex::new(std::collections::HashMap::new()));
    let user_table_clone = Arc::clone(&user_table);

    println!("Starting mDNS service thread...");

    // Listen for Services, Respond & Store
    std::thread::spawn(move || {
        while let Ok(event) = receiver.recv() {
            match event {
                mdns_sd::ServiceEvent::ServiceResolved(info) => {
                    println!("Service resolved: {:?}", info);
                    // Send request to create tcp connection
                    let addresses = info.get_addresses_v4();
                    for address in addresses {
                        let mut user_table_clone = user_table_clone.lock().unwrap();
                        let user_socket = format!("{}:{}", address, info.get_port());
                        match std::net::TcpStream::connect(&user_socket){
                            Ok(stream) => {
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
                            },
                            Err(e) => println!("Failed to connect to user {}: {}", user_socket, e),
                        }
                    }
                },
                _ => {
                    println!("Unhandled mDNS event");

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

            let message = format!("{}: {}", instance_name, message);
            let stream = match stream.try_clone() {
                Ok(stream) => stream,
                Err(e) => {
                    println!("Failed to clone stream for {}: {}", username, e);
                    continue;
                }
            };
            let mut stream = stream.try_clone().unwrap();
            match stream.write(message.as_bytes()) {
                Ok(_) => println!("Sent message to {}: {}", username, message),
                Err(e) => println!("Failed to send message to {}: {}", username, e),
            }
<<<<<<< HEAD
            stream.write(message.as_bytes()).unwrap();
            
            // Something must refresh the terminal every second or so clear out the display
            // fetch the information from the data structure containing the streams and user names and print it
            // Clear out the buffer 
=======
            stream.write(message.as_bytes()).expect("Unable to send message");
>>>>>>> refs/remotes/origin/main
        }
    }
}

