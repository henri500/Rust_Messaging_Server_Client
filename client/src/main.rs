use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError, Sender};
use std::thread;
use std::str::from_utf8;

use aes::Aes256;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 6969;
const IV: &[u8; 16] = b"5551234567890145";

fn connection_handling() -> TcpStream {
    let client =
        TcpStream::connect(LOCAL)
            .unwrap();//("Stream failed to connect");
    client
        .set_nonblocking(true)
        .expect("failed to initiate non-blocking");
    return client
}

fn write_message(tx: &Sender<String>) -> String {
    loop {
        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("reading from stdin failed");
        let msg =
            buff
                .trim()
                .to_string();
        if msg == "" { continue }
        else if msg == ":quit" || tx.send(String::from(&msg)).is_err() {
            panic!("Cannot send message, exiting program...")
        } else { return msg }
    }
}

fn handle_incoming_msg(client: &mut TcpStream, first_co: &mut bool, key: &Vec<u8>) -> bool{
    let mut buff = vec![0; MSG_SIZE];
    match client.read_exact(&mut buff) {
        Ok(_) => {
            let msg =
                buff
                    .into_iter()
                    .collect::<Vec<_>>();
            let msg = decrypt_msg_aes(msg, key);
            match from_utf8(&msg)  {
                Ok(data) => {
                    if !*first_co {
                        println!("-> {}", data);
                    } else {
                        println!("Connected to -> {} | as -> {}", LOCAL, data);
                        *first_co = false;
                    }
                }
                Err(err) => {
                    println!("Error reading the data : {:?}", err);
                }
            };
        },

        Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
        Err(_) => {
            println!("connection with server was severed");
            return true;
        }
    }
    return false;
}

fn send_message(server: &mut TcpStream, mut msg: Vec<u8>) {
    msg.resize(MSG_SIZE, 0);
    server
        .write_all(&msg)
        .expect("writing to socket failed");
}

fn encrypt_message(msg: Vec<u8>, key: &Vec<u8>) -> Vec<u8>{
    type Aes256Cbc = Cbc<Aes256, Pkcs7>;

    let cipher = match Aes256Cbc::new_from_slices(&key, IV) {
        Ok(cipher) => cipher,
        Err(err) => panic!("{}", err)
    };

    let mut buffer = [0u8; MSG_SIZE];
    buffer[.. msg.len()].copy_from_slice(msg.as_slice());

    let enc_data = match cipher.encrypt(&mut buffer, msg.len()) {
        Ok(enc_data) => enc_data.to_vec(),
        Err(err) => {
            println!("Could not encrypt message : {:?}", err);
            return b"".to_vec();
        }
    };
    return enc_data.to_owned();
}

fn unpacking(data: &mut Vec<u8>) -> Vec<u8> {
    // Used to remove the zeros at the end of the received encrypted message
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

fn decrypt_msg_aes(mut msg: Vec<u8>, key: &Vec<u8>) -> Vec<u8> {
    type Aes256Cbc = Cbc<Aes256, Pkcs7>;
    let new_iv =  b"5551234567890145".to_vec();
    let x = new_iv.as_slice();
    let cipher = match Aes256Cbc::new_from_slices(&key, &x) {
        Ok(cipher) => cipher,
        Err(err) => panic!("{}", err)
    };

    msg = unpacking(&mut msg);
        match cipher.clone().decrypt(&mut msg) {
            Ok(decrypted_data) => {
                return decrypted_data.to_vec();
            },
            Err(_) => {
                    println!("Wrong Password. Try again:");
                    return b"".to_vec();
            }
    }
}

fn encrypt_and_send_message(server: &mut TcpStream, msg: Vec<u8>, key: &Vec<u8>) {
    let enc_pass = encrypt_message(msg,  &key);
    send_message(server, enc_pass);
}

fn send_pass(server: &mut TcpStream) -> Vec<u8> {
    let mut server_pass_and_key = String::new();

    println!("Server's password : ");
    io::stdin()
        .read_line(&mut server_pass_and_key)
        .expect("Failed to to get input");
    server_pass_and_key.pop();

    let enc_pass = encrypt_message(server_pass_and_key.clone().into_bytes(),  &server_pass_and_key.clone().into_bytes(), );

    send_message(server, enc_pass);
    return server_pass_and_key.into_bytes().to_owned();
}

fn main() {
    let mut server = connection_handling();
    let mut first_co: bool = true;
    let (tx, rx) = mpsc::channel::<String>();
    let key = send_pass(&mut server);

    thread::spawn(move ||
        loop {
            let disconnect = handle_incoming_msg(&mut server, &mut first_co, &key);
            if disconnect { break }
            match rx.try_recv() {
                Err(TryRecvError::Empty) => ( continue ),
                Ok(msg) => {
                    encrypt_and_send_message(&mut server, msg.into_bytes(), &key, );
                },
                Err(TryRecvError::Disconnected) => { break }
            }
        });
    loop {
        write_message(&tx);
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn check_iv_len() {
        assert_eq!(super::IV.len(), 16);
    }

}