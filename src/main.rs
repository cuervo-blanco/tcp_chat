use std::net::{TcpStream, UdpSocket};
use std::io::{Write, Read};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
#[allow(unused_imports)]
use std::time::Duration;
#[allow(unused_imports)]
use audio_sync::audio;
use std::sync::mpsc::channel;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::thread;


// This here macro below me was written 
// by chatgpt... it works...
// (for now) 
// *play disaster music* 
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

#[tokio::main]
async fn main () {
    // Initial Position
    clear_terminal();
    debug_println!("MAIN: AUDIO INITIALIZATION IN PROCESS");
    let (Some(input_device), Some(output_device)) = audio::initialize_audio_interface() else {
            debug_println!("MAIN: AUDIO INITIALIZATION FAILED");
        return;
    };
    let input_config = audio::get_audio_config(&input_device)
        .expect("Failed to get audio input config");
    debug_println!("MAIN: INPUT AUDIO CONFIG: {:?}", input_config);
    let output_config = audio::get_audio_config(&output_device)
        .expect("Failed to get audio output config");
    debug_println!("MAIN: OUTPUT AUDIO CONFIG: {:?}", output_config);
    let input_device = Arc::new(Mutex::new(input_device));
    let input_config = Arc::new(Mutex::new(input_config));

    let audio_buffer = Arc::new(Mutex::new(Vec::new()));
    debug_println!("MAIN Allocated audio buffer: {:?}", audio_buffer);
    let received_data = Arc::clone(&audio_buffer);

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

    let name = instance_name.clone();
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

                let message = vec![name.to_string(), input.to_string(), TERMINATOR.to_string()].join(SEPARATOR);
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
    let tcp_socket_addr = format!("{}:{}", ip, port);
    // Open TCP port 18521 (listen to connections)
    let listener = std::net::TcpListener::bind(tcp_socket_addr.clone())
        .expect("Failed to bind listener");

    //---- The UDP Thread -----//

    let udp_port: u16 = 18522;
    let udp_socket_addr = format!("{}:{}", ip, udp_port);
    let udp_socket = Arc::new(Mutex::new(UdpSocket::bind(udp_socket_addr.clone()).unwrap()));
    let mut buf = [0; 960];
    udp_socket.lock().unwrap().recv_from(&mut buf).expect("UDP: Nothing to receive");
    let ip_table: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(std::collections::HashMap::new()));
    let binding = ip_table.clone();
    let ip_table_clone = binding.lock().unwrap();
    // ADD CONDITIONAL CALL (WAIT FOR INPUT FROM USER)
    for (_user, ip) in ip_table_clone.iter() {
        debug_println!("UDP: Connected to {} on {}", _user, ip);
        udp_socket.lock().unwrap().connect(ip).unwrap();
    };
    let received_data_clone = Arc::clone(&received_data);
    let udp_socket_clone = Arc::clone(&udp_socket);
    // Handle incoming audio
    thread::spawn( move || {
        let udp_socket = udp_socket_clone.clone();
        let mut buffer = [0; 960];
        loop {
            match udp_socket.lock().unwrap().recv(&mut buffer) {
                Ok(size) => {
                    let mut data = received_data_clone.lock().unwrap();
                    data.extend_from_slice(&buffer[..size]);
                }
                Err(e) => {
                    eprintln!("Failed to receive data: {}", e);
                }
            }
        }
    });

    #[allow(unused_variables)]
    let output_stream = audio::start_output_stream(
        &output_device,
        &output_config,
        received_data.clone()
    ).expect("Failed to start output stream");
    
    // ---- Sending Audio - Read User Input ----//
    let (tx, rx) = channel();
    thread::spawn(move || {
        loop {
            if event::poll(Duration::from_millis(100)).unwrap() {
                if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap(){
                    tx.send(code).unwrap();
                }

            }
        }
    });

    let udp_socket_clone_2 = Arc::clone(&udp_socket);
    let ip_table_clone_2 = Arc::clone(&ip_table);
    thread::spawn( move || {
        let mut input_stream = None;
        loop {
            match rx.recv().unwrap() {
                KeyCode::F(1) => {
                    debug_println!("crossterm: f1: start input stream");
                    if input_stream.is_none() {
                        let (stream, receiver) = 
                            audio::start_input_stream(
                                &input_device.lock().unwrap(), 
                                &input_config.lock().unwrap()
                            )
                            .expect("Failed to start input stream");
                        
                        let receiver = Arc::new(Mutex::new(receiver));
                        input_stream = Some((stream, Arc::clone(&receiver)));
                        let udp_socket_2 = Arc::clone(&udp_socket_clone_2);
                        let ip_table = Arc::clone(&ip_table_clone_2);
                         thread::spawn(move || {
                            loop {
                                // Handle encoded data (await)
                                if let Ok(opus_data) = receiver.lock().unwrap().recv() {
                                    let slice: &[u8] = &opus_data;
                                    let udp_socket_2 = udp_socket_2.lock().unwrap();
                                    let ip_table_2 = ip_table.lock().unwrap();
                                    for (user, ip) in ip_table_2.iter() {
                                        debug_println!("UDP: Sending audio to {} on {}", user, ip);
                                        if *user == hostname::get().unwrap().to_str().unwrap().to_string() {
                                            continue;
                                        }
                                        udp_socket_2.send_to(slice, ip).expect("Failed to send data");
                                    };
                                }
                            }
                        });
                    }
                }, 
                KeyCode::F(2) => {
                    if let Some((stream, _)) = input_stream.take() {
                        println!("F2 pressed: Stop Input Stream");
                        audio::stop_audio_stream(stream);
                    }

                },
                _ => println!("Pressed a different key"),

            }
        }
    });

    debug_println!("THREAD 2: Starting TCP stream reader and text generator thread...");
    std::thread::spawn( move || {
        debug_println!("THREAD 2: Succesful Deployment of Thread.");
        loop {
            debug_println!("THREAD 2: Entering Thread Loop.");
            for tcp_stream in listener.incoming() {
                debug_println!("THREAD 2: Listening for incoming packets.");
                match tcp_stream {
                    Ok(stream) => {
                            let mut stream = stream.try_clone().unwrap();
                            debug_println!("THREAD 2: Stream Cloned <{:?}>", stream);
                            std::thread::spawn(move || {
                                let mut buffer = [0; 512];
                                debug_println!("THREAD 2: Memory allocated to buffer: {:?}", buffer);
                                loop {
                                    match stream.read(&mut buffer) {
                                        Ok(bytes_read) => {
                                            if bytes_read == 0 {
                                            break;
                                            }

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
                                            let msg: String = match String::from_utf8(filtered_data) {
                                                Ok(s) => {
                                                    debug_println!("THREAD 2: Converted string: {}", s);
                                                    s
                                                },
                                                Err(e) => { 
                                                    eprintln!("THREAD 2: Failed to convert bytes to string: {}", e);
                                                    continue;
                                                }
                                            };

                                            debug_println!("THREAD 2: Message: {:?}", msg);
                                            let pos = msg.find(TERMINATOR).unwrap_or(msg.len());
                                            debug_println!("THREAD 2: Finding position of message_end: {:?}", pos);
                                            let text = &msg[..pos];
                                            debug_println!("THREAD 2: Splitting message: {}", text);

                                            // Update the positions 
                                            // Send the header to the print thread
                                            debug_println!("THREAD 2: Converting message to string: {}", text);
                                            let mut spread = Vec::new();
                                            debug_println!("THREAD 2: Allocating Memory for received message: {:?}", spread);

                                            if let Some(slash_index) = text.find(SEPARATOR) {
                                                let username: &str = &text[..slash_index];
                                                spread.push(username.to_string());
                                                let message: &str = &text[slash_index + 1..text.len() - 1];
                                                spread.push(message.to_string());
                                            }
                                            debug_println!("THREAD 2: Cleaning up message: {:?}", spread);
                                            // Print message on screen
                                            if !spread.is_empty() {
                                                println!("{}", spread.join(": "));
                                            }
                                        },
                                        Err(e) => {
                                            eprintln!("Failed to read from stream: {}", e);
                                            break;
                                        }
                                    }
                                }
                            });
                        },
                        Err(e) => println!("Error getting stream: {}", e),
                    }
                }
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
                            let user_udp_socket = format!("{}:18522", address);
                            let user_hostname = info.get_hostname();
                            let mut ip_table = ip_table.lock().expect("THREAD 3: Failed to lock ip_table");
                            ip_table.insert(
                                user_hostname.to_string(), 
                                user_udp_socket
                                );
                            // --------- Tcp Connection ---------//
                            match std::net::TcpStream::connect(&user_socket){
                                Ok(stream) => {
                                    user_table.insert(info.get_fullname().to_string(), stream);
                                    
                                    debug_println!("THREAD 3: Inserted New User into User Table: {:?}", user_table_clone);
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
