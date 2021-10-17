use std::{io::{self, Write}, net::TcpStream};

fn main() {
    let mut stream = TcpStream::connect("localhost:8888").expect("Could not connect to server");
    
    loop {
        let mut message = String::new();
        //let mut buffer: Vec<u8> = Vec::new();

        //println!("Write a message to send:");
        io::stdin().read_line(& mut message).expect("Failed to read from stdin"); //message a entrer par l utilisateur
        stream.write(message.as_bytes()).expect("Failed To write to server"); //ecrit le message dans le stream -> envoie le message au serveur
    }
}
