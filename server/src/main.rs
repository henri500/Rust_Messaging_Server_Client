use std::{io::{Error, Read, Write}, net::{TcpListener, TcpStream}, thread};

fn handle_client(mut stream: TcpStream) -> Result<(), Error> {
    println!("\nIncoming connection from: {}", stream.peer_addr()?);
    
    let mut buffer = [0; 12];
    let mut message: Vec<u8> = Vec::new();

    loop {
        let nb_message_bytes= stream.read(&mut buffer)?; //lis le message qui est dans le stream dans le buffer -> recupere le message du client
        //println!("NB_MESSAGES_BYTES: {}", nb_message_bytes);
        //println!("BUFFER: {:?}", &buffer[..nb_message_bytes]);
        
        if nb_message_bytes == 0 { //lorsque le client se deconnecte, il envoie un message vide
            println!("Deconnection from: {}", stream.peer_addr()?);
            return Ok(()); //pour éviter la boucle infinie
        }

        for byte in buffer.iter() {
            if *byte != '\n' as u8 {
                message.push(*byte);
            }
        }

        println!("Message received from {:} : {}", stream.peer_addr()?, String::from_utf8(message).unwrap());
        
        //stream.write(&buffer[..nb_message_bytes])?; //ecrit dans le stream, ecrit au client (meme client, meme message)

        message = Vec::new();
        buffer = [0; 12];
    }
}
fn main() {
    let listener = TcpListener::bind("localhost:8888").expect("Could not bind");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream).unwrap();
                });
            }
            Err(e) => {
                println!("Failed: {}", e)
            }
        }
    }
}
