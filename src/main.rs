use std::net::TcpStream;
use std::io::{Write, Read};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
#[allow(unused_imports)]
use std::time::Duration;

#[allow(unused_attributes)]
#[macro_use]
macro_rules! debug_println {
    ($($arg:tt)*) => (
        #[cfg(feature = "debug")]
        println!($($arg)*)
        )
}

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
    debug_println!("MAIN: Data Structures Initialized");

    // -------- Input Thread ------- //
    std::thread::spawn ( move || {
        debug_println!("THREAD 1: Thread Initialized");
        loop {
            // Take user input
            let reader = std::io::stdin();
            let mut buffer: String = String::new();
            reader.read_line(&mut buffer).unwrap();
            let input = buffer.trim();
            debug_println!("THREAD 1: User Input: {}", input);

            let user_table = user_table.lock().unwrap();
            debug_println!("THREAD 1: User Table Lock: {:?}", user_table);
            for (user, stream) in user_table.iter() {
                // Clean up the name, get rid of .local
                let username: String = user.split('.').next().unwrap_or("").to_string();
                debug_println!("THREAD 1: Sending message to: {:?}", username);

                let message = vec![username.to_string(), input.to_string(), TERMINATOR.to_string()].join(SEPARATOR);
                debug_println!("THREAD 1: Message to Send: {:?}", message);
                let encoded_message: Vec<u8> = bincode::serialize(&message).unwrap();
                debug_println!("THREAD 1: Encoded message: {:?}", encoded_message);

                // Verify if this accessing of the operation is valid
                debug_println!("THREAD 1: Verifying stream: {:?}", stream);
                match stream.try_clone() {
                    Ok(mut stream) => {
                        match stream.write(&encoded_message) {
                            Ok(_) => {
                                debug_println!("THREAD 1: Successfully send message to {}", user);
                            }
                            Err(e) => {
                                eprintln!("THREAD 1: Failed to send message to {}: {}", user, e);
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("THREAD 1: Failed to clone stream for {}: {}", username, e);
                        continue;
                    }
                };
                // Something must refresh the terminal every second or so clear out the display
                // fetch the information from the data structure containing the streams and user names and print it
                // Clear out the buffer 
                debug_println!("THREAD 1: RESTARTING LOOP #1");
            }
             debug_println!("THREAD 1: RESTARTING MAIN LOOP");
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

    debug_println!("THREAD 2: Thread Initializing Parameters");
    // Get information from local host to start tcp stream
    let ip =  local_ip_address::local_ip().unwrap();
    let port: u16 = 18521;
    let socket_addr = format!("{}:{}", ip, port);
    // Open TCP port 18521 (listen to connections)
    let listener = std::net::TcpListener::bind(socket_addr.clone())
        .expect("Failed to start listener");

    debug_println!("THREAD 2: Starting TCP stream reader and text generator thread...");
    std::thread::spawn( move || {
        debug_println!("THREAD 2: Succesful Deployment of Thread.");
        loop {
            debug_println!("THREAD 2: Entering Thread Loop.");
            for tcp_stream in listener.incoming() {
                debug_println!("THREAD 2: Listening for incoming packets.");
                match tcp_stream {
                    Ok(stream) => {
                        loop {
                            let mut stream = stream.try_clone().unwrap();
                            debug_println!("THREAD 2: Stream Cloned <{:?}>", stream);
                            let mut buffer = [0; 512];
                            debug_println!("THREAD 2: Memory allocated to buffer: {:?}", buffer);

                            let bytes_read = stream.read(&mut buffer).unwrap();
                            debug_println!("THREAD 2: Incoming Bytes_Read: {:?}", &buffer[..bytes_read]);
                            let incoming_message = &buffer[..bytes_read];
                            debug_println!("THREAD 2: Incoming Message: {:?}", incoming_message);

                            let mut data = Vec::new();
                            debug_println!("THREAD 2: Allocating Memory (DATA) for Incoming Message: data {:?}", data);

                            data.extend_from_slice(incoming_message);
                            debug_println!("THREAD 2: Saving buffer: data {:?}", data);
                            let filtered_data: Vec<u8> = data
                                .into_iter()
                                .filter(|&b| b.is_ascii_graphic() || b.is_ascii_whitespace())
                                .collect();
                            let mut msg: String = match String::from_utf8(filtered_data) {
                                Ok(s) => {
                                    debug_println!("THREAD 2: Converted string: {}", s);
                                    s
                                },
                                Err(e) => { 
                                    eprintln!("THREAD 2: Failed to convert bytes to string: {}", e);
                                    std::process::exit(1);
                                    
                                }
                            };
                            
                                debug_println!("THREAD 2: Message: {:?}", msg);
                                fn find_position(string: &String, end_mark: char) -> usize {
                                    if let Some(pos) = string.chars().position(|b| b == end_mark){
                                        return pos
                                    } else {
                                        debug_println!(" THREAD 2: Failed to get position");
                                        std::process::exit(2);
                                    }
                                }
                                let pos = find_position(&msg, '\n');
                                debug_println!("THREAD 2: Finding position of message_end: {:?}", pos);
                                let _text = msg.split_off(pos + 1);
                                debug_println!("THREAD 2: Splitting message: {}", _text);

                                // Update the positions 
                                // Send the header to the print thread
                                let string: String = msg.to_string();
                                debug_println!("THREAD 2: Converting message to string: {}", string);
                                let mut spread = Vec::new();
                                debug_println!("THREAD 2: Allocating Memory for received message: {:?}", spread);

                                if let Some(slash_index) = string.find(SEPARATOR) {
                                    let username: &str = &string[..slash_index];
                                    spread.push(username.to_string());
                                    let message: &str = &string[slash_index + 1..string.len() - 2];
                                    spread.push(message.to_string());
                                }
                                debug_println!("THREAD 2: Cleaning up message: {:?}", spread);
                                // Print message on screen
                                println!("{}", spread.join(": "));
                        
                        }
                    }
                    Err(e) => println!("Error getting stream: {}", e),
                }
            }
            debug_println!("THREAD 2: Restarting first loop");
        }
    });

    // ----------- mDNS Service Thread ----------//
    
    // Configure Service
    debug_println!("MAIN: Commencing mDNS Service");
    let mdns = mdns_sd::ServiceDaemon::new().expect("Failed to create daemon");
    let service_type = "_tcp_chat._tcp.local.";
    let ip =  local_ip_address::local_ip().unwrap();
    debug_println!("MAIN: Connecting to Local IP address: {}", ip);
    let host_name =  hostname::get()
        .expect("MAIN: Unable to get host name");
    let host_name = host_name.to_str()
        .expect("MAIN: Unable to convert to string");
    let host_name = format!("{}.local.", host_name);
    debug_println!("MAIN: Host name: {}", host_name);
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
    debug_println!("MAIN: Service Info created: {:?}", tcp_chat_service);

    // Broadcast service
    mdns.register(tcp_chat_service).expect("Failed to register service");
    debug_println!("MAIN: Service registered");

    // Query for Services
    let receiver = mdns.browse(service_type).expect("Failed to browse");
    debug_println!("MAIN: Browsing for services: {:?}", receiver);



    debug_println!("THREAD 3: Starting mDNS service thread...");
    // Listen for Services, Respond & Store
    loop {
        debug_println!("THREAD 3: Starting Thread 3 loop");
            while let Ok(event) = receiver.recv() {
                match event {
                    mdns_sd::ServiceEvent::ServiceResolved(info) => {
                        debug_println!("THREAD 3: Service resolved: {:?}", info);
                        // Send request to create tcp connection
                        let addresses = info.get_addresses_v4();
                        debug_println!("THREAD 3: Addresses found: {:?}", addresses);
                        for address in addresses {
                            let mut user_table = user_table_clone.lock().unwrap();
                            let user_socket = format!("{}:{}", address, info.get_port());
                            debug_println!("THREAD 3: User Socket: {:?}", user_socket);
                            // --------- Tcp Connection ---------//
                            match std::net::TcpStream::connect(&user_socket){
                                Ok(stream) => {
                                    user_table.insert(info.get_fullname().to_string(), stream);
                                    debug_println!("THREAD 3: Inserted New User into User Table: {:?}", user_table_clone);
                                    let mut username = String::new();
                                    debug_println!("THREAD 3: Username: {:?}", username);
                                    for char in info.get_fullname().chars() {
                                        if char != '.' {
                                            username.push(char);
                                        } else {
                                            break;
                                        }
                                    }
                                    debug_println!("{} just connected", username);
                                },
                                Err(e) => eprintln!("Failed to connect to user {}: {}", user_socket, e),
                            }
                        }
                    },
                    _ => {

                    }
                }
            }
            debug_println!("THREAD 3: Restarting Loop");
    }
    // Optional: Show Services Discovered
}

