use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

fn main() {
    let listener = TcpListener::bind("localhost:8888").expect("Could not bind");
    listener.set_nonblocking(true).expect("failed to initialize non-blocking");

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<String>();

    loop {
        match listener.accept() {
            Ok((mut stream, addr)) => { //cas ou un client se connecte au serveur
                println!("Incoming connection from: {}", addr);
                
                let tx = tx.clone();
                
                clients.push(stream.try_clone().expect("Failed to clone"));
                
                thread::spawn(move || loop {
                    let mut buffer = [0; 32]; //va contenir le message recu du client, celui-ci peut contenir que 32 caracteres max
                    
                    match stream.read(&mut buffer) {
                        Ok(0) => { //deconnexion du client
                            println!("Deconnection from: {}", addr);
                            break;
                        },
                        Ok(_) => { //contient toutes les autres expressions, cas ou il y a un message dans le buffer
                            let mut message = Vec::new();

                            for byte in buffer.iter() { //copier strictement que le message
                                if *byte != 0 {
                                    message.push(*byte);
                                }
                            }

                            let message = String::from_utf8(message).expect("Invalid utf8 message");
    
                            println!("Message received from {}: {:?}", addr, message);
                            tx.send(message).expect("Failed to send message to rx");
                        }
                        Err(_) => {}
                    }
                });
            }         
            Err(_) => {}
        }

        match rx.try_recv() {
            Ok(message) => { //cas ou le client recoit le message envoye
                for mut client in clients.iter() {
                    let mut buffer = message.clone().into_bytes();

                    buffer.resize(32, 0); 
                    client.write(&buffer).expect("Writing to socket failed"); 
                }
            }
            Err(_) => {}
        }
    }
}