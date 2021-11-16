use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self};
use std::thread;

fn main() {
    let mut client = TcpStream::connect("localhost:8888").expect("Stream failed to connect");
    client.set_nonblocking(true).expect("Failed to initiate non-blocking");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buffer = [0; 32]; //va contenir le message recu, celui-ci peut contenir que 32 caracteres max

        match client.read(&mut buffer) {
            Ok(0) => { //deconnexion du serveur
                println!("Connection with server was lost");
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

                println!("Message received: {:?}", message);
            }
            Err(_) => {}
        }

        match rx.try_recv() { 
            Ok(message) => { //cas ou le client recoit un message envoye
                let mut buffer = message.clone().into_bytes();

                buffer.resize(32, 0);
                client.write(&buffer).expect("Writing to socket failed");
                println!("Message sent: {:?}", message);
            }, 
            Err(_) => {},
        }
    });

    println!("Write a Message:");
    loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).expect("Failed to read from stdin"); //client ecrit un message a envoyer
        let message = buffer.trim().to_string(); //enlever le caractere "\n" a la fin du message
        tx.send(message);
    }
}