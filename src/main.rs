use std::io::{Write, Read};
use std::sync::{Arc, Mutex, Condvar};
use std::time::Duration;

//---Definitions---//
fn  clear_terminal() {
    print!("\x1B[2J");
    std::io::stdout().flush().unwrap();
}
fn username_take()-> String {
    // Take user input (instance name)
    let reader = std::io::stdin();
    let mut instance_name = String::new();
    reader.read_line(&mut instance_name).unwrap();
    let instance_name = instance_name.replace("\n", "").replace(" ", "_");
    instance_name
}
fn  move_cursor(row: u32, col: u32) {
    print!("\x1B[{};{}H", row, col);
    std::io::stdout().flush().unwrap();
}
fn demultiplex(text: String)-> Vec<String> {
    let mut spread = Vec::new();
    if let Some(slash_index) = text.find("/#t/") {
        let username: &str = &text[..slash_index];
        spread.push(username.to_string());
        let message: &str = &text[slash_index + 4..];
        spread.push(message.to_string());
    }
    spread
}
fn write_at_position(row: u32, col: u32, text: String) {
    move_cursor(row, col);
    print!("{} ", text);
    std::io::stdout().flush().unwrap();
}
type Position = (u32, u32);

fn update_position(positions: Vec<Position>) -> Vec<Position>{
    let mut updated_positions = Vec::new();
    for (x, y) in positions {
        updated_positions.push((x+1, y));
    }
    updated_positions
}

fn main () {
    // Initial Position
    clear_terminal();

    println!("");
    println!("Enter Username:");
    // Add validation process? 
    #[allow(unused_variables)]
    let instance_name = username_take();

    println!("System preparing for take off...");
    let first_column: Position = (1 as u32, 1 as u32);
    let second_column: Position = (1 as u32, 20 as u32);
    let positions: Vec<Position> = vec![first_column, second_column];
    let shared_positions =  Arc::new((Mutex::new(positions.clone()), Condvar::new()));

    #[allow(unused_mut)]
    let mut header = demultiplex("username/#t/message".to_string()); 
    // Equivalent to the header is the message, it's meant to carry the message 
    // and pass it along the threads
    #[allow(unused_mut)]
    let mut message = Arc::new((Mutex::new(Vec::<String>::new()), Condvar::new()));

    for (i, (row, col)) in positions.iter().enumerate() {
        write_at_position(*row, *col, header[i].clone());
    }    

    // A thread to receive the bytes coming from a tcp stream, 
    // convert the byte stream into a String (with a termination symbol
    // or null character to signify the end of the message of a user
    // the messages are going to be strutured as such:
    // sender/#t/message/n
    // with /n being the escape character
    // with t standig for text and e for end
    // When it packages the info it passes it to the print thread
    // that will update the current position and print the provided strings
    
    //------ The Printing Thread -----------//
    // Start the print message thread and make it wait
    let sp = Arc::clone(&shared_positions); 
    let msg = Arc::clone(&message);
    std::thread::spawn( move || {
        // Taking the shared positions borrowed in order to make changes to it
        let (p_lock, pcvar) = &*sp;
        let mut positions = p_lock.lock().unwrap();
        // Borrowing the message String to store the incoming message there
        let (m_lock, mcvar) = &*msg;
        let mut incoming_message = m_lock.lock().unwrap();

        loop {
            positions = pcvar.wait(positions).unwrap();
            incoming_message = mcvar.wait(incoming_message).unwrap();
            // Capture header and positions before doing this:
            for (i, (row, col)) in positions.iter().enumerate() {
                write_at_position(*row, *col, incoming_message[i].clone());
            }    
        }
        
    });

    //---- The TCP Thread -----//

    // Get information from local host to start tcp stream
    let ip =  local_ip_address::local_ip().unwrap();
    let port: u16 = 18521;
    let socket_addr = format!("{}:{}", ip, port);
    // Open TCP port 18521 (listen to connections)
    let listener = std::net::TcpListener::bind(socket_addr.clone())
        .expect("Failed to start listener");

    println!("Starting TCP stream reader and text generator thread...");
    let sp = Arc::clone(&shared_positions); 
    let msg = Arc::clone(&message);
    std::thread::spawn( move || {
        for tcp_stream in listener.incoming() {
            match tcp_stream {
                Ok(stream) => {
                    let mut stream = stream.try_clone().unwrap();
                    let mut buffer = [0; 512];
                    let bytes_read = stream.read(&mut buffer).unwrap();
                    if bytes_read == 0 {
                        break;
                    }

                    let (p_lock, pcvar) = &*sp;
                    let mut positions = p_lock.lock().unwrap();
                    let (m_lock, mcvar) = &*msg;
                    let mut incoming_message = m_lock.lock().unwrap();

                    let mut data = Vec::new();
                    data.extend_from_slice(&buffer[..bytes_read]);
                    if let Some(pos) = data.iter().position(|&b| b == b'\n') {
                        let text = data.split_off(pos + 1);

                        if let Ok(string) = std::str::from_utf8(&text) {
                            // Update the positions 
                            // Send the header to the print thread
                            let string: String = string.to_string();
                            let mut spread = Vec::new();

                            if let Some(slash_index) = string.find("/#t/") {
                                let username: &str = &string[..slash_index];
                                spread.push(username.to_string());
                                let message: &str = &string[slash_index + 4..];
                                spread.push(message.to_string());
                            }

                            *incoming_message = spread;
                            *positions = update_position(positions.to_vec());

                            pcvar.notify_one();
                            mcvar.notify_one();
                            
                        }
                        
                    }
                    
                }
                Err(e) => println!("Error getting stream: {}", e),
            }
        }
    });

    // ----------- mDNS Service Thread ----------//
    
    // Configure Service
    let mdns = mdns_sd::ServiceDaemon::new().expect("Failed to create daemon");
    let service_type = "_tcp_chat._tcp.local.";
    let ip =  local_ip_address::local_ip().unwrap();
    println!("Connecting to Local IP address: {}", ip);
    let host_name =  hostname::get()
        .expect("Unable to get host name");
    let host_name = host_name.to_str()
        .expect("Unable to convert to string");
    let host_name = format!("{}.local.", host_name);
    println!("Host name: {}", host_name);
    let properties = [("property_1", "attribute_1"), ("property_2", "attribute_2")];

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
                        // --------- Tcp Connection ---------//
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

                }
            }
        }
    });

    // Optional: Show Services Discovered

    // -------- Input Thread ------- //
    loop {
        // Take user input
        let reader = std::io::stdin();
        let mut buffer: String = String::new();
        reader.read_line(&mut buffer).unwrap();
        let user_table = user_table.lock().unwrap();
        let input = buffer.trim();
        // Send message to each socket upon user input Enter
        // Write to the streams in those sockets
        for (user, stream) in user_table.iter() {
            // Clean up the name, get rid of .local
            let mut username = String::new();
            for char in user.chars() {
                if char != '.' {
                    username.push(char);
                } else {
                    break;
                }

            }

            let message = vec![username.to_string(), input.to_string()];
            let message = message.join("#/t/");
            let encoded_message: Vec<u8> = bincode::serialize(&message).unwrap();

            // Verify if this accessing of the operation is valid
            let stream = match stream.try_clone() {
                Ok(stream) => stream,
                Err(e) => {
                    println!("Failed to clone stream for {}: {}", username, e);
                    continue;
                }
            };
            let mut stream = stream.try_clone().unwrap();
            // For testing purposes send message written at different intervals
            // of time
            loop {
                stream.write(&encoded_message).unwrap();
                std::thread::sleep(Duration::from_millis(2000));
            }
            // Something must refresh the terminal every second or so clear out the display
            // fetch the information from the data structure containing the streams and user names and print it
            // Clear out the buffer 
        }
    }


}

