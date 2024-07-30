use std::net::TcpStream;
use std::io::{Write, Read};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
#[allow(unused_imports)]
use std::time::Duration;

const SEPARATOR: &str = "#";
const TERMINATOR: &str = "\n";

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
#[allow(dead_code)]
fn demultiplex(text: String)-> Vec<String> {
    let mut spread = Vec::new();
    if let Some(slash_index) = text.find(SEPARATOR) {
        let username: &str = &text[..slash_index];
        spread.push(username.to_string());
        let message: &str = &text[slash_index + 1..];
        spread.push(message.to_string());
    }
    spread
}

fn main () {
    // Initial Position
    clear_terminal();

    println!("");
    println!("Enter Username:");
    // Add validation process? 
    #[allow(unused_variables)]
    let instance_name = username_take();
    clear_terminal();

    println!("System preparing for take off...");
    std::thread::sleep(std::time::Duration::from_millis(1000));
    clear_terminal();

    println!("");
    // User table to store users discovered and their service information
    let user_table: Arc<Mutex<HashMap<String, TcpStream>>> = Arc::new(Mutex::new(std::collections::HashMap::new()));
    let user_table_clone = Arc::clone(&user_table);
    println!("MAIN: Data Structures Initialized");

    // -------- Input Thread ------- //
    std::thread::spawn ( move || {
        println!("THREAD 1: Thread Initialized");
        loop {
            // Take user input
            let reader = std::io::stdin();
            let mut buffer: String = String::new();
            reader.read_line(&mut buffer).unwrap();
            let input = buffer.trim();
            println!("THREAD 1: User Input: {}", input);

            let user_table = user_table.lock().unwrap();
            println!("THREAD 1: User Table Lock: {:?}", user_table);
            // Send message to each socket upon user input Enter
            // Write to the streams in those sockets
            for (user, stream) in user_table.iter() {
                // Clean up the name, get rid of .local
                let mut username = String::new();

                println!("THREAD 1: Found Username: {:?}", username);
                for char in user.chars() {
                    if char != '.' {
                        username.push(char);
                    } else {
                        break;
                    }

                }
                println!("THREAD 1: Display Username: {:?}", username);

                let message = vec![username.to_string(), input.to_string(), TERMINATOR.to_string()];
                println!("THREAD 1: Message Vector: {:?}", message);
                let message = message.join(SEPARATOR);
                println!("THREAD 1: Message to send: {:?}", message);
                let encoded_message: Vec<u8> = bincode::serialize(&message).unwrap();
                println!("THREAD 1: Encoded message: {:?}", encoded_message);

                // Verify if this accessing of the operation is valid
                println!("THREAD 1: Verifying stream: {:?}", stream);
                match stream.try_clone() {
                    Ok(mut stream) => {
                        if let Err(e) = stream.write_all(&encoded_message) {
                            eprintln!("THREAD 1: Failed to send message to {}: {}", user, e);
                        }
                    },
                    Err(e) => {
                        println!("THREAD 1: Failed to clone stream for {}: {}", username, e);
                        continue;
                    }
                };
                // Something must refresh the terminal every second or so clear out the display
                // fetch the information from the data structure containing the streams and user names and print it
                // Clear out the buffer 
                println!("THREAD 1: RESTARTING LOOP #1");
            }
             println!("THREAD 1: RESTARTING MAIN LOOP");
        }
    });

    // A thread to receive the bytes coming from a tcp stream, 
    // convert the byte stream into a String (with a termination symbol
    // or null character to signify the end of the message of a user
    // the messages are going to be strutured as such:
    // sender/#t/message/n
    // with /n being the escape character
    // with t standig for text and e for end
    // When it packages the info it passes it to the print thread
    // that will update the current position and print the provided strings

    //---- The TCP Thread -----//

    println!("THREAD 2: Thread Initializing Parameters");
    // Get information from local host to start tcp stream
    let ip =  local_ip_address::local_ip().unwrap();
    let port: u16 = 18521;
    let socket_addr = format!("{}:{}", ip, port);
    // Open TCP port 18521 (listen to connections)
    let listener = std::net::TcpListener::bind(socket_addr.clone())
        .expect("Failed to start listener");

    println!("THREAD 2: Starting TCP stream reader and text generator thread...");
    std::thread::spawn( move || {
        println!("THREAD 2: Succesful Deployment of Thread.");
        loop {
            println!("THREAD 2: Entering Thread Loop.");
            for tcp_stream in listener.incoming() {
                println!("THREAD 2: Listening for incoming packets.");
                match tcp_stream {
                    Ok(stream) => {
                        let mut stream = stream.try_clone().unwrap();
                        println!("THREAD 2: Stream Cloned <{:?}>", stream);
                        let mut buffer = [0; 512];
                        println!("THREAD 2: Memory allocated to buffer: {:?}", buffer);

                        let bytes_read = stream.read(&mut buffer).unwrap();
                        println!("THREAD 2: Incoming Bytes_Read: {:?}", &buffer[..bytes_read]);
                        let incoming_message = &buffer[..bytes_read];
                        println!("THREAD 2: Incoming Message: {:?}", incoming_message);

                        let mut data = Vec::new();
                        println!("THREAD 2: Allocating Memory (DATA) for Incoming Message: data {:?}", data);

                        data.extend_from_slice(&buffer[..bytes_read]);
                        println!("THREAD 2: Saving buffer: data {:?}", data);
                        let filtered_data: Vec<u8> = data
                            .into_iter()
                            .filter(|&b| b.is_ascii_graphic() || b.is_ascii_whitespace())
                            .collect();
                        let mut msg: String = match String::from_utf8(filtered_data) {
                            Ok(s) => {
                                println!("THREAD 2: Converted string: {}", s);
                                s
                            },
                            Err(e) => { 
                                eprintln!("THREAD 2: Failed to convert bytes to string: {}", e);
                                std::process::exit(1);
                                
                            }
                        };
                        
                            println!("THREAD 2: Message: {:?}", msg);
                            fn find_position(string: &String, end_mark: char) -> usize {
                                if let Some(pos) = string.chars().position(|b| b == end_mark){
                                    return pos
                                } else {
                                    println!(" THREAD 2: Failed to get position");
                                    std::process::exit(2);
                                }
                            }
                            let pos = find_position(&msg, '\n');
                            println!("THREAD 2: Finding position of message_end: {:?}", pos);
                            let text = msg.split_off(pos + 1);
                            println!("THREAD 2: Splitting message: {}", text);

                            // Update the positions 
                            // Send the header to the print thread
                            let string: String = msg.to_string();
                            println!("THREAD 2: Converting message to string: {}", string);
                            let mut spread = Vec::new();
                            println!("THREAD 2: Allocating Memory for received message: {:?}", spread);

                            if let Some(slash_index) = string.find(SEPARATOR) {
                                let username: &str = &string[..slash_index];
                                spread.push(username.to_string());
                                let message: &str = &string[slash_index + 1..string.len() - 2];
                                spread.push(message.to_string());
                            }
                            println!("THREAD 2: Cleaning up message: {:?}", spread);
                            println!("{}", spread.join(" "));
                            
                        
                    }
                    Err(e) => println!("Error getting stream: {}", e),
                }
            }
            println!("THREAD 2: Restarting first loop");
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


    println!("Starting mDNS service thread...");

    // Listen for Services, Respond & Store
    std::thread::spawn(move || {
        loop {
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
        }
    }).join().unwrap();

    // Optional: Show Services Discovered

}

