use std::{io::{self, Write, BufReader, BufRead}, net::TcpStream};
use std::str;

fn main() {
    let mut stream = TcpStream::connect("localhost:8888").expect("Could not connect to server");
    
    loop {
        let mut message = String::new();
        let mut buffer: Vec<u8> = Vec::new();

        //println!("Write a message to send:");
        io::stdin().read_line(& mut message).expect("Failed to read from stdin"); //message a entrer par l utilisateur
        stream.write(message.as_bytes()).expect("Failed To write to server"); //ecrit le message dans le stream -> envoie le message au serveur

        /*let mut reader = BufReader::new(&stream);
        reader.read_until(b'\n', &mut buffer).expect("Could not read into buffer"); //lis le buffer, contient ce que le serveur a envoyé (meme message du client)
        println!("{}", str::from_utf8(&buffer).expect("Could not write buffer as string"));*/
    }
}
