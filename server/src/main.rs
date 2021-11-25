use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::channel;
use std::thread::{spawn};

use aes::Aes256;

use block_modes::{BlockMode,Cbc,block_padding::Pkcs7};

const LOCAL: &str = "127.0.0.1:6000";
const MAX_MSG_SIZE: usize = 6969;
const SERVER_PASSWD: &[u8; 32] = b"12345678901234567890123556789011";
const IV: &[u8; 16] = b"5551234567890145";

struct User {
    ip: String,
    data: Vec<u8>,
    authenticated: bool,
}

fn connection_handling() -> TcpListener{
    let server =
        TcpListener::bind(LOCAL)
            .unwrap();
    server
        .set_nonblocking(true)
        .expect("failed to initialize non-blocking");
    return server;
}

fn register_client(mut clients: Vec<TcpStream>, new_user: &TcpStream) -> Vec<TcpStream> {
    // add the newly connected client to the array

    clients.push(new_user
        .try_clone()
        .expect("failed to clone client")
    );
    return clients;
}

fn broadcast_msg(clients: &Vec<TcpStream>, u: &mut User) {
    // Used to send a message to all other clients

    for mut c in clients{
        if u.ip != c.peer_addr().unwrap().to_string() {
            u.data.resize(MAX_MSG_SIZE, 0);
            c.write_all(&u.data).ok();
        };
    };
}

fn send_message_to_client(client: &mut TcpStream, u: &mut User) {
    u.data.resize(MAX_MSG_SIZE, 0);
    client.write_all(&u.data).ok();
}

fn handling_incoming_msg(socket: &mut TcpStream, addr: &SocketAddr) -> (Vec<u8>, bool) {
    let mut buff = vec![0; MAX_MSG_SIZE];

    match socket.read_exact(&mut buff) {
        Ok(_) => {
            let msg = buff.into_iter().collect::<Vec<_>>();
            return (msg, false);
        },

        Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
        Err(x) => {
            println!("Connection closed with: {}, {}", addr, x);
            return (buff, true);
        }
    }
    return (buff, true);
}


fn unpacking(data: &mut Vec<u8>) -> Vec<u8> {
    let mut transit:Vec<u8> = vec![];
    let mut res:Vec<u8> = vec![];
    let mut keep_push = false;
    for d in data.iter().rev() {
        if *d == 0 && !keep_push{
            continue;
        } else {
            transit.push(*d);
            keep_push = true;
        }
    }
    for t in transit.iter().rev() {
        res.push(*t);
    }
    return res.to_owned();
}

fn msg_crypt(msg: Vec<u8>, key: &Vec<u8>) -> Vec<u8>{
    type Aes256Cbc = Cbc<Aes256, Pkcs7>; // Aliasing for. similar to import <module> as in python

    let cipher = match Aes256Cbc::new_from_slices(&key, IV) {
        Ok(cipher) => cipher,
        Err(err) => panic!("{}", err)
    };
    let mut buffer = [0u8; MAX_MSG_SIZE]; // creating buffer to hold message:
    buffer[.. msg.len()].copy_from_slice(msg.as_slice());

    let enc_data = match cipher.encrypt(&mut buffer, msg.len()) {
        Ok(enc_data) => enc_data.to_vec(),
        Err(err) => {
            println!("Error while decrypting the data: {:?}", err);
            return b"".to_vec(); // return and empty vector of bytes
        }
    };
    return enc_data.to_owned();
}

fn msg_decrypt(mut msg: Vec<u8>, key: &Vec<u8>) -> (Vec<u8>, bool) {

    type Aes256Cbc = Cbc<Aes256, Pkcs7>;
    let new_iv =  b"5551234567890145".to_vec();
    let x = new_iv.as_slice();
    let cipher = match Aes256Cbc::new_from_slices(&key,&x) {
        Ok(cipher) => cipher,
        Err(err) => {
            println!("Decryption error --> {}", err);
            return (b"".to_vec(), false)
        }
    };
    msg = unpacking(&mut msg); // Getting rid of trailing zeros
    match cipher.decrypt(&mut msg) {
        Ok(decrypted_data) => {
            return (decrypted_data.to_vec(), true);
        }
        Err(err) => {
            println!("Could not decrypt message -> {:?}", err);
            return (b"".to_vec(), false);
        }
    };
}

fn main() {
    println!("Server listening on .... {}", LOCAL);
    let server = connection_handling();
    let mut clients = vec![];
    let mut authenticated = false;
    let (tx, rx) = channel::<User>();

    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            let tx = tx.clone();
            clients = register_client(clients, &socket);

            spawn(move || loop {
                let mut u = User{
                    ip: socket.peer_addr().unwrap().to_string(),
                    data: vec![],
                    authenticated,
                };// retaining User data
                let server_password = SERVER_PASSWD.to_vec();
                if !u.authenticated {
                    let (data, _) = handling_incoming_msg(&mut socket, &addr);
                    let (_, auth_passed) = msg_decrypt(data, &server_password);
                    if auth_passed {
                        println!("New client: {} connected and successfully authenticated", addr);

                        let mut greetings_msg = socket.peer_addr().unwrap().to_string().into_bytes();
                        greetings_msg.extend_from_slice(b"\nSuccessfully authenticated");
                        u.data = msg_crypt(greetings_msg, &server_password);
                        send_message_to_client(&mut socket, &mut u);
                        authenticated = true;
                        continue;
                    } else {
                        println!("Client {} sent a wrong password", addr);
                        u.data = b"Message from server :\n\tIncorect password, please try again".to_vec();
                        send_message_to_client(&mut socket, &mut u);
                        break;
                    }
                }

                let (msg , disconnect) = handling_incoming_msg(&mut socket, &addr);

                if !disconnect {
                    let (msg, _) = msg_decrypt(msg, &server_password);
                    u.data = msg;
                    match tx.send(u) {
                        Ok(_) => {},
                        Err(err) => {
                            println!("Error while sending message to channel : {:?}", err);
                        }
                    }
                } else { break };
            });
        }
        if let Ok(mut u) = rx.try_recv() {
            u.data = msg_crypt(u.data.to_owned(), &SERVER_PASSWD.to_vec());
            broadcast_msg(&clients, &mut u);
        };
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn check_pass_len() {
        assert_eq!(super::SERVER_PASSWD.len(), 32);
    }
    #[test]
    fn check_iv_len() {
        assert_eq!(super::IV.len(), 16);
    }

}