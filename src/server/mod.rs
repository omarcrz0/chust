
use std::{net::{TcpListener, TcpStream, Shutdown, SocketAddr}, io::{Write, Read}, sync::{Arc, Mutex}, collections::HashMap, ops::Deref, str::from_utf8};
// use std::str::from_utf8;
use std::{thread, time::Duration};

use crate::packet::Packet;

// enum Signal {
//     Kill
// }

// struct ThreadChannel {
//     sender: mpsc::Sender<Signal>,
//     receiver: Arc<Mutex<mpsc::Receiver<Signal>>>
// }

struct ClientConnection { 
    socket: Arc<TcpStream>,
    // threads: Vec<thread::JoinHandle<()>>
    // thread_channel: ThreadChannel
}

#[derive(Clone)]
pub struct Server {
    client_connections: Arc<Mutex<HashMap<SocketAddr, ClientConnection>>>,
}

impl Server {
    pub fn new() -> Server {
        Server { 
            client_connections: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn start(&mut self) {
        // Make server have its own scope.
        let listener = TcpListener::bind("127.0.0.1:6000").unwrap();

        for socket in listener.incoming() {
            // Get socket stream else panic
            let socket = socket.unwrap();
            let addr = socket.peer_addr().unwrap();
            
            self.new_client(socket, &addr);
            
            self.client_listener(&addr);
            self.user_connection_verifier(&addr);
        }
    }

    fn print_number_of_clients(&self) {
        let data = self.client_connections.lock().unwrap().len();
        println!("Total Clients: {data}\n")
    }

    fn new_client(&mut self, socket: TcpStream, addr: &SocketAddr) {
        // Get socket Address
        println!("\n+++ New client at: {addr:?}");
        let client_connection = ClientConnection {
            socket: Arc::new(socket),
        };

        self.client_connections.lock().unwrap().insert(addr.clone(), client_connection);
        let message = format!("+{} user connected.", addr);
        //TODO: MAKE THIS MESSAGE A PACKET IN ORDER TO BE SENT.

        for (_addr, client) in self.client_connections.lock().unwrap().iter_mut() {
            let socket_ptr = client.socket.clone();
            let mut temp_socket = socket_ptr.deref();
            // temp_socket.write(message.as_bytes()).unwrap();
        }

        self.print_number_of_clients();

    }

    fn remove_client(&mut self, addr: SocketAddr) {
        self.client_connections.lock().unwrap().remove(&addr);
        let message = format!("-{} user disconnected.", addr);
        for (_addr, client) in self.client_connections.lock().unwrap().iter_mut() {
            let socket_ptr = client.socket.clone();
            let mut temp_socket = socket_ptr.deref();
            //TODO: MAKE THIS MESSAGE A PACKET IN ORDER TO BE SENT.
            // temp_socket.write(message.as_bytes()).unwrap();
        }

        println!("\n--- Removed client at: {addr:?}");
        self.print_number_of_clients();
    }

    fn client_listener(&mut self, addr: &SocketAddr) {
        // TODO: This method needs to make use of Packet to parse received packets sent by clients
        let client_connections_lock = self.client_connections.lock().unwrap();
        let client_connection = client_connections_lock.get(addr).unwrap();
        let socket_ptr =  client_connection.socket.clone();

        thread::spawn(move || {
            let mut socket = socket_ptr.deref();
            let mut data = [0 as u8; 1024];
            'reading_data: while match socket.read(&mut data) {
                Ok(n) => {
                    if n == 0 { 
                        break 'reading_data; 
                    }
                    let stringified_data = from_utf8(&data).unwrap();
                    println!("Message sent from client: {stringified_data}");
                    true
                },
                Err(_) => {
                    false
                }
            } {} // while statement is the last {}
        });
    }

    fn user_connection_verifier(&mut self, addr: &SocketAddr) {
        let mut this = self.clone();
        let client_connections_lock = self.client_connections.lock().unwrap();
        let client_connection = client_connections_lock.get(addr).unwrap();
        let socket_ptr =  client_connection.socket.clone();

        thread::spawn(move || {
            let mut socket = socket_ptr.deref();
            let local_address = socket.peer_addr().unwrap();

            let message = b"{\"action\": \"ping\"}".to_vec();
            let packet = Packet::prepare_packet(message);

            while 
            match socket.write(&packet) {
                Ok(_) => {
                    println!("LOGGER: PING SENT.");
                    thread::sleep(Duration::from_secs(5));
                    true
                }
                Err(_) => {
                    socket.shutdown(Shutdown::Both).expect("shutdown call failed");
                    this.remove_client(local_address);
                    false
                }
            } { } // while statement is the last {}
        });
    }
}
