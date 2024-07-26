#[allow(unused_imports)]
use std::io::{Write, Read};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};


fn handle_client(mut stream: std::net::TcpStream, position: &Position) {
    let mut buffer: &[u8] = &[0; 960];
    match stream.read(&mut buffer) {
        Ok(data) => {
            let message: Vec<String> = bincode::deserialize(&buffer).unwrap();
            if data == 0 {
                println!("Connection closed by the client");
                return;
            }
            for (i, &(row, col)) in position.iter().enumerate() {
                write_at_position(row, col, &message[i]);
            }    
        }
        Err(e) => println!("Error reading byte stream: {}", e),
    }
}

fn  move_cursor(row: u32, col: u32) {
    print!("\x1B[{};{}H", row, col);
    std::io::stdout().flush().unwrap();
}

fn  clear_terminal() {
    print!("\x1B[2J");
    std::io::stdout().flush().unwrap();
}

fn write_at_position(row: u32, col: u32, text: &str) {
    move_cursor(row, col);
    print!("{} ", text);
    std::io::stdout().flush().unwrap();
}


#[allow(dead_code)]
struct Connection {
    user: String,
    stream: TcpStream,
}

fn update_position(positions: &Position) -> Position {
    for (row, col) in positions.iter().enumerate() {
        row += 1;
    }
}

type Message<'a> = [&'a str];
type Position = [(u32, u32)];

fn main () {
    clear_terminal();
    // Initial Position
    let mut positions: &Position = &[(1, 1), (1, 20)];
    let texts: &Message = &["Username", "Message"];

    for (i, &(row, col)) in positions.iter().enumerate() {
        write_at_position(row, col, texts[i]);
    }    

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
                    let new_positions = update_position(positions);
                    handle_client(stream, &new_positions)
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
        let message = [];
        let reader = std::io::stdin();
        let mut buffer: String = String::new();
        reader.read_line(&mut buffer).unwrap();
        let user_table = user_table.lock().unwrap();
        let input = buffer.trim();
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

            let message = [&username, &input.to_string(), "0"];
            let encoded_message = bincode::serialize(&message).unwrap();

            let stream = match stream.try_clone() {
                Ok(stream) => stream,
                Err(e) => {
                    println!("Failed to clone stream for {}: {}", username, e);
                    continue;
                }
            };
            let mut stream = stream.try_clone().unwrap();
            stream.write(&encoded_message).unwrap();
            // Something must refresh the terminal every second or so clear out the display
            // fetch the information from the data structure containing the streams and user names and print it
            // Clear out the buffer 
        }
    }
}

