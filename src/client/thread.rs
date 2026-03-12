/* 
Copyright (c) 2026  NickelAnge.Studio 
Email               mathieu.grenier@nickelange.studio
Git                 https://github.com/NickelAngeStudio/ethos-client

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use std::{io::{Read, Write}, net::TcpStream, sync::mpsc::{Receiver, Sender}};

use ethos_core::net::{CLIENT_MSG_MAX_SIZE, ClientMessage, MESSAGE_SIZE_TYPE_SIZE, SERVER_MSG_BUFFER_SIZE, ServerMessage};
use crate::{Error as ClientError, EthosClient, EthosClientStatus};

use crate::{ EthosClientUpdate, client::message::{CtoTMessage, StoCMessage}};

/// Client thread parameters
pub(crate) struct ClientThread {
    pub(crate)  connect_string : String, 
    pub(crate)  rcv_ctot : Receiver<CtoTMessage>, 
    pub(crate)  sdr_ttoc : Sender<EthosClientUpdate>, 
    pub(crate)  sdr_stoc : Sender<StoCMessage>,
    pub(crate)  status : EthosClientStatus,
    pub(crate)  inc_size : Option<usize>
}

impl EthosClient {

    
    pub(super) fn handle_client_thread(mut ct : ClientThread) {

        // Put buffer on heap
        let mut buffer = Vec::<u8>::new();
        buffer.resize(SERVER_MSG_BUFFER_SIZE, 0);

        match TcpStream::connect(ct.connect_string.clone()){
            Ok(mut stream) => {

                match stream.set_nonblocking(true) {
                    Ok(_) => {
                        // Set status as connected
                        ct.status = EthosClientStatus::Connected;
                        Self::send_update(&mut ct, EthosClientUpdate::StatusChanged(EthosClientStatus::Connected));

                        'active:
                        loop {
                            if let EthosClientStatus::Connected = ct.status {
                                Self::handle_client_message(&mut stream, &mut ct, &mut buffer);
                            } else {
                                break 'active;
                            }

                            if let EthosClientStatus::Connected = ct.status {
                                Self::handle_server_message(&mut stream, &mut ct, &mut buffer);
                            } else {
                                break 'active;
                            }
                        }

                        // Shutdown stream
                        match stream.shutdown(std::net::Shutdown::Both){
                            Ok(_) => {},
                            Err(err) => Self::send_update(&mut ct, EthosClientUpdate::Error(ClientError::UnhandledIOError(err.kind()))),
                        }
                    },
                    Err(err) => Self::send_update(&mut ct, EthosClientUpdate::Error(ClientError::UnhandledIOError(err.kind()))),
                }
            },
            Err(err) => { 
                match err.kind() {
                    // Invalid connect string
                    std::io::ErrorKind::InvalidInput => Self::send_update(&mut ct, EthosClientUpdate::Error(ClientError::InvalidConnectionString)),
                    // Server is down / busy
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::ConnectionRefused | 
                    std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::HostUnreachable | std::io::ErrorKind::NetworkUnreachable | 
                    std::io::ErrorKind::ConnectionAborted  | std::io::ErrorKind::NotConnected => {
                        Self::send_update(&mut ct, EthosClientUpdate::Error(ClientError::ServerDown));
                    }
                    // Other IO error
                    _ => Self::send_update(&mut ct, EthosClientUpdate::Error(ClientError::UnhandledIOError(err.kind()))),
                }
                
            },
        }

    }

    /// Send an update to client. If it fail, it will disconnect thread instead of panic!.
    #[inline]
    fn send_update(ct : &mut ClientThread, update : EthosClientUpdate) {

        #[cfg(debug_assertions)]
        {
            match &update { // Print error in debug mode
                EthosClientUpdate::Error(_) => println!("{:?}", update),
                _ => {},
            }
        }

        match ct.sdr_ttoc.send(update){
            Ok(_) => {},
            Err(_) => { // Channel is closed, communication to client is lost, disconnect thread
                ct.status = EthosClientStatus::Disconnecting;
                Self::send_update(ct, EthosClientUpdate::StatusChanged(EthosClientStatus::Disconnecting));
            },
        }

    }
    

    /// Handle messages coming from EthosClient.
    #[inline]
    fn handle_client_message(stream : &mut TcpStream, ct : &mut ClientThread, buffer : &mut Vec<u8>){

        'client:
        loop {
            match ct.rcv_ctot.try_recv() {
                Ok(message) => match message {
                    CtoTMessage::SendMessage(client_message) => Self::handle_client_message_send(stream, ct, client_message, buffer),
                    CtoTMessage::CloseConnection => Self::handle_client_message_close(ct),
                },
                Err(_) => break 'client,
            }
        }
        
    }

    /// Send message to server
    #[inline]
    fn handle_client_message_send(stream : &mut TcpStream, ct : &mut ClientThread, client_message : ClientMessage, buffer : &mut Vec<u8>){

        if client_message.size as usize <= CLIENT_MSG_MAX_SIZE {    // Limit size of message

            match client_message.pack_bytes(buffer) {
                Ok(size) => {
                    if client_message.size as usize <= CLIENT_MSG_MAX_SIZE { 
                        match stream.write_all(&buffer[0..size]) {
                            Ok(_) => {},
                            Err(err) => Self::send_update(ct, EthosClientUpdate::Error(ClientError::ClientMessageSendError(err.kind()))),
                        }
                    } else {    // Message too large to send
                        Self::send_update(ct, EthosClientUpdate::Error(ClientError::ClientMessageTooLarge));
                    }
                },
                Err(err) => Self::send_update(ct, EthosClientUpdate::Error(ClientError::ClientMessagePackError(err))),
            }

        } else {    // Message is too large to send
            Self::send_update(ct, EthosClientUpdate::Error(ClientError::ClientMessageTooLarge));
        }


    }

    /// Close client connection to server
    #[inline]
    fn handle_client_message_close(ct : &mut ClientThread){

        ct.status = EthosClientStatus::Disconnecting;
        Self::send_update(ct, EthosClientUpdate::StatusChanged(EthosClientStatus::Disconnecting));


    }

    /// Handle message coming from server
    #[inline]
    fn handle_server_message(stream : &mut TcpStream, ct : &mut ClientThread, buffer : &mut Vec<u8>) {

        // Read the size first if any
        Self::handle_server_message_size(stream, ct, buffer);
        // Read the message if size found
        if let Some(size) = ct.inc_size {
            Self::handle_server_message_read(stream, ct, buffer, size);
        } 

    }

    /// Handle reading size of message
    #[inline]
    fn handle_server_message_size(stream : &mut TcpStream, ct : &mut ClientThread, buffer : &mut Vec<u8>) {

        if ct.inc_size.is_none() {
            // Read size first if none
            ct.inc_size = match stream.read_exact(&mut buffer[..MESSAGE_SIZE_TYPE_SIZE]) {
                Ok(_) => Some(ServerMessage::size_from_bytes(&buffer[0..MESSAGE_SIZE_TYPE_SIZE].try_into().unwrap()) as usize),
                Err(err) => match err.kind() {
                    std::io::ErrorKind::WouldBlock => None,
                    _ => {  // Report read error
                        Self::send_update(ct, EthosClientUpdate::Error(ClientError::ServerMessageReadError(err.kind())));
                        None
                    },
                },
            };
        }

    }

    /// Handle reading the message itself
    fn handle_server_message_read(stream : &mut TcpStream, ct : &mut ClientThread, buffer : &mut Vec<u8>, size:usize) {

        match stream.read_exact(&mut buffer[..size]) {
            Ok(_) => match ServerMessage::from_bytes(&buffer[0..size]){
                Ok(message) => {
                    Self::send_server_message_to_client(ct, message);
                    ct.inc_size = None;
                },  // Error shouldn't happens since message are coming from server via TCP.
                Err(err) => Self::send_update(ct, EthosClientUpdate::Error(ClientError::ServerMessageFromBytesError(err))),
            },
            Err(err) => match err.kind() {
                std::io::ErrorKind::WouldBlock => {},   // Try later
                _ => Self::send_update(ct, EthosClientUpdate::Error(ClientError::ServerMessageReadError(err.kind()))),  // Report read error
            } 

            
        };

    }

    /// Send the server message received to client via channel
    #[inline]
    fn send_server_message_to_client(ct : &mut ClientThread, msg : ServerMessage) {

         match ct.sdr_stoc.send(StoCMessage::Message(msg)) {
            Ok(_) => {},
            Err(_) => { // Channel is closed, communication to client is lost, disconnect thread
                ct.status = EthosClientStatus::Disconnecting;
                Self::send_update(ct, EthosClientUpdate::StatusChanged(EthosClientStatus::Disconnecting));
            },
        }

    }

}