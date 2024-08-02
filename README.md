# tcp_chat

## Overview
This module facilitates local communication between devices on the same network. The primary goal is to establish reliable communication within the same physical space, emphasizing local interaction without relying on external internet connectivity.

### Features
- **Text Messaging**: Allows users to send and receive text messages.
- **Control Messaging**: Future updates will include control messages for enhanced interaction.
- **Local Network Communication**: Focused on communication within the same local network.
- **Service Discovery**: Utilizes mDNS for discovering and connecting to services.

## Installation

### Prerequisites
- [Install Rust](https://www.rust-lang.org/tools/install)

### Running the Application
1. Clone the repository:
    ```sh
    git clone https://github.com/cuervo-blanco/tcp_chat.git
    cd tcp_chat
    ```

2. Build and run the application using Cargo:
    ```sh
    cargo run
    ```

## Code Overview
Here's a brief overview of what the code does:

1. **User Input Handling**: Takes the username input from the user and prepares the system for communication.
2. **Message Broadcasting**: A thread is spawned to handle user input and broadcast messages to all connected users.
3. **TCP Listener**: Another thread listens for incoming TCP connections and handles incoming messages, printing them to the terminal.
4. **mDNS Service Discovery**: The module uses mDNS to broadcast its service and discover other services on the local network, facilitating automatic connection to other users.

### Main Components
- **Input Thread**: Continuously reads user input and sends messages to connected peers.
- **TCP Thread**: Listens for incoming TCP connections and processes received messages.
- **mDNS Service Thread**: Handles service discovery and registration using mDNS, allowing devices to find each other on the local network.

## Future Updates
This module is expected to receive regular updates. Future versions will support control messages and additional features to enhance its utility. It will be integrated into the broader walkie-talkie-app, offering a robust solution for local communication needs.

## Contributing
Contributions are welcome. Please fork the repository and submit a pull request with your changes.

