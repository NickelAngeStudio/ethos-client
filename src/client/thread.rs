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

use debug_print::debug_eprintln;
use ethos_core::net::{CLIENT_MSG_MAX_SIZE, ClientMessage, MESSAGE_SIZE_TYPE_SIZE, SERVER_MSG_BUFFER_SIZE, ServerMessage};
use crate::{Error as ClientError, EthosClientStatus};

use crate::{ EthosClientUpdate, client::message::{CtoTMessage, StoCMessage}};

/// Client thread parameters
pub(crate) struct ClientThread {
    channels : ClientThreadChannel,
    status : EthosClientStatus,
    buffer : Vec<u8>,
    inc_size : Option<usize>
}

/// Thread communication channels of [ClientThread]
pub(crate) struct ClientThreadChannel {
    rcv_ctot : Receiver<CtoTMessage>, 
    sdr_ttoc : Sender<EthosClientUpdate>, 
    sdr_stoc : Sender<StoCMessage>,
}

impl ClientThread {
    /// Create a new [ClientThread] from channels.
    pub(crate) fn new(rcv_ctot : Receiver<CtoTMessage>, sdr_ttoc : Sender<EthosClientUpdate>, sdr_stoc : Sender<StoCMessage>) -> ClientThread {
        ClientThread { channels: ClientThreadChannel { rcv_ctot, sdr_ttoc, sdr_stoc }, 
            status: EthosClientStatus::Disconnected, inc_size: None, buffer : Vec::<u8>::new() }
    }


    /// [ClientThread] execution routine.
    pub(super) fn execute(&mut self) {

        // Resize buffer on heap
        self.buffer.resize(SERVER_MSG_BUFFER_SIZE, 0);

        'main:
        loop {
            match self.status {
                EthosClientStatus::Ended => break 'main,
                EthosClientStatus::Disconnected => {
                    // Wait for connection
                    match self.channels.rcv_ctot.recv() {
                        Ok(message) => {
                            match message {
                                CtoTMessage::OpenConnection(connect_string) => self.handle_connection(connect_string),
                                CtoTMessage::EndClient => break 'main,
                                _ => self.send_update(EthosClientUpdate::Error(ClientError::ClientDisconnected)), 
                            }
                        },
                        Err(_) => break 'main,  // Channel to EthosClient lost
                    }
                },
                _ => {  // Client is now disconnected
                    self.status = EthosClientStatus::Disconnected;
                    self.send_update(EthosClientUpdate::StatusChanged(EthosClientStatus::Disconnected));
                }
            }
        }

        // Tell client that thread ended
        self.send_update(EthosClientUpdate::StatusChanged(EthosClientStatus::Ended));

    }

    /// Handle connection of client thread
    #[inline]
    fn handle_connection(&mut self, connect_string : String) {

        match TcpStream::connect(connect_string){
                Ok(mut stream) => {

                    match stream.set_nonblocking(true) {
                        Ok(_) => {
                            match stream.set_nodelay(true) {    // disables the Nagle algorithm
                                Ok(_) => self.handle_stream(&mut stream),
                                Err(err) => self.send_update(EthosClientUpdate::Error(ClientError::UnhandledIOError(err.kind()))),
                            }
                            
                        },
                        Err(err) => self.send_update(EthosClientUpdate::Error(ClientError::UnhandledIOError(err.kind()))),
                    }
                },
                Err(err) => { 
                    match err.kind() {
                        // Invalid connect string
                        std::io::ErrorKind::InvalidInput => self.send_update(EthosClientUpdate::Error(ClientError::InvalidConnectionString)),
                        // Server is down / busy
                        std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::ConnectionRefused | 
                        std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::HostUnreachable | std::io::ErrorKind::NetworkUnreachable | 
                        std::io::ErrorKind::ConnectionAborted  | std::io::ErrorKind::NotConnected => {
                            self.send_update(EthosClientUpdate::Error(ClientError::ServerDown));
                        }
                        // Other IO error
                        _ => self.send_update( EthosClientUpdate::Error(ClientError::UnhandledIOError(err.kind()))),
                    }
                    
                },
            }

    }

    /// Client thread active loop routine
    #[inline]
    fn handle_stream(&mut self, stream : &mut TcpStream) {

         // Put buffer on heap
        let mut buffer = Vec::<u8>::new();
        buffer.resize(SERVER_MSG_BUFFER_SIZE, 0);

        // Set status as connected
        self.status = EthosClientStatus::Connected;
        self.send_update( EthosClientUpdate::StatusChanged(EthosClientStatus::Connected));

        'stream:
        loop {
            if let EthosClientStatus::Connected = self.status {
                self.handle_client_message(stream);
            } else {
                break 'stream;
            }

            if let EthosClientStatus::Connected = self.status {
                self.handle_server_message(stream);
            } else {
                break 'stream;
            }
        }

        // Shutdown stream
        match stream.shutdown(std::net::Shutdown::Both){
            Ok(_) => {},
            Err(err) => self.send_update(EthosClientUpdate::Error(ClientError::UnhandledIOError(err.kind()))),
        }

    }


    /// Send an update to client. If it fail, it will disconnect thread instead of panic!.
    #[inline]
    fn send_update(&mut self, update : EthosClientUpdate) {

        #[cfg(debug_assertions)]
        {
            match &update { // Print error in debug mode
                EthosClientUpdate::Error(_) => println!("{:?}", update),
                _ => {},
            }
        }

        match self.channels.sdr_ttoc.send(update){
            Ok(_) => {},
            Err(_) => { // Channel is closed, communication to client is lost, end thread
                debug_eprintln!("send_update sdr_ttoc channel down!");
                self.status = EthosClientStatus::Ended;
            },
        }

    }
    

    /// Handle messages coming from EthosClient.
    #[inline]
    fn handle_client_message(&mut self, stream : &mut TcpStream){

        'client:
        loop {
            match self.channels.rcv_ctot.try_recv() {
                Ok(message) => match message {
                    CtoTMessage::SendMessage(client_message) => self.handle_client_message_send(stream, client_message),
                    CtoTMessage::CloseConnection => self.handle_client_message_close(),
                    CtoTMessage::OpenConnection(_) => self.send_update(EthosClientUpdate::Error(ClientError::ClientAlreadyConnected)),
                    CtoTMessage::EndClient => {
                        self.status = EthosClientStatus::Ended;
                        break 'client;
                    },
                },
                Err(_) => break 'client,
            }
        }
        
    }

    /// Send message to server
    #[inline]
    fn handle_client_message_send(&mut self, stream : &mut TcpStream, client_message : ClientMessage){

        if client_message.size as usize <= CLIENT_MSG_MAX_SIZE {    // Limit size of message

            match client_message.pack_bytes(&mut self.buffer) {
                Ok(size) => {
                    if client_message.size as usize <= CLIENT_MSG_MAX_SIZE { 
                        match stream.write_all(&mut self.buffer[0..size]) {
                            Ok(_) => {},
                            Err(err) => self.send_update( EthosClientUpdate::Error(ClientError::ClientMessageSendError(err.kind()))),
                        }
                    } else {    // Message too large to send
                        self.send_update( EthosClientUpdate::Error(ClientError::ClientMessageTooLarge));
                    }
                },
                Err(err) => self.send_update( EthosClientUpdate::Error(ClientError::ClientMessagePackError(err))),
            }

        } else {    // Message is too large to send
            self.send_update( EthosClientUpdate::Error(ClientError::ClientMessageTooLarge));
        }


    }

    /// Close client connection to server
    #[inline]
    fn handle_client_message_close(&mut self){

        self.status = EthosClientStatus::Disconnecting;
        self.send_update( EthosClientUpdate::StatusChanged(EthosClientStatus::Disconnecting));


    }

    /// Handle message coming from server
    #[inline]
    fn handle_server_message(&mut self, stream : &mut TcpStream) {

        // Read the size first if any
        self.handle_server_message_size(stream);
        // Read the message if size found
        if let Some(size) = self.inc_size {
            self.handle_server_message_read(stream, size);
        } 

    }

    /// Handle reading size of message
    #[inline]
    fn handle_server_message_size(&mut self, stream : &mut TcpStream) {

        if self.inc_size.is_none() {
            // Read size first if none
            self.inc_size = match stream.read_exact(&mut self.buffer[..MESSAGE_SIZE_TYPE_SIZE]) {
                Ok(_) => Some(ServerMessage::size_from_bytes(&mut self.buffer[0..MESSAGE_SIZE_TYPE_SIZE].try_into().unwrap()) as usize),
                Err(err) => match err.kind() {
                    std::io::ErrorKind::WouldBlock => None,
                    _ => {  // Report read error and close connection
                        self.send_update( EthosClientUpdate::Error(ClientError::ServerMessageReadError(err.kind())));
                        self.handle_client_message_close();
                        None
                    },
                },
            };
        }

    }

    /// Handle reading the message itself
    fn handle_server_message_read(&mut self, stream : &mut TcpStream, size:usize) {

        match stream.read_exact(&mut self.buffer[..size]) {
            Ok(_) => match ServerMessage::from_bytes(&mut self.buffer[0..size]){
                Ok(message) => {
                    self.send_server_message_to_client(message);
                    self.inc_size = None;
                },  // Error shouldn't happens since message are coming from server via TCP.
                Err(err) => self.send_update( EthosClientUpdate::Error(ClientError::ServerMessageFromBytesError(err))),
            },
            Err(err) => match err.kind() {
                std::io::ErrorKind::WouldBlock => {},   // Try later
                _ => { // Report read error and close connection
                    self.send_update( EthosClientUpdate::Error(ClientError::ServerMessageReadError(err.kind())));
                    self.handle_client_message_close();
                }, 
            } 

            
        };

    }

    /// Send the server message received to client via channel
    #[inline]
    fn send_server_message_to_client(&mut self, msg : ServerMessage) {

         match self.channels.sdr_stoc.send(StoCMessage::Message(msg)) {
            Ok(_) => {},
            Err(_) => { // Channel is closed, communication to client is lost, end thread
                debug_eprintln!("send_server_message_to_client sdr_stoc channel down!");
                self.status = EthosClientStatus::Ended;
            },
        }

    }

}