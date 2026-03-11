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

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

use ethos_core::net::{CLIENT_MSG_MAX_SIZE, ClientMessage, MESSAGE_SIZE_TYPE_SIZE, ServerMessage};

use crate::timeout_loop;

/// Small server used for unit tests.
pub(crate) struct UnitTestServer {
    listener : Option<TcpListener>,
    tcp_stream : Option<TcpStream>,
    inc_msg_size : Option<usize>,
    buffer : [u8;CLIENT_MSG_MAX_SIZE]
}

impl UnitTestServer {
    pub fn new() -> UnitTestServer {
        UnitTestServer { listener: None, tcp_stream: None, inc_msg_size: None, buffer: [0u8; CLIENT_MSG_MAX_SIZE] }
    }

    /// Start the test server
    pub fn start(&mut self, connect_string : String) {
        self.listener = Some(TcpListener::bind(connect_string).unwrap());
        self.listener.as_mut().unwrap().set_nonblocking(true).unwrap();
    }

    /// Close the server
    pub fn close(&mut self) {

        // Close stream
        match self.tcp_stream.take().unwrap().shutdown(std::net::Shutdown::Both){
            Ok(_) => {}, // Prevent panic!
            Err(_) => {},
        }

        // Close listener
        self.listener.take().unwrap();

    }

    /// Accept connection and open stream
    /// 
    /// Will panic if loop expires.
    pub fn accept(&mut self) {
        timeout_loop!{
            for stream in self.listener.as_mut().unwrap().incoming() {
                match stream {
                    Ok(stream) => {
                        stream.set_nonblocking(true).unwrap();
                        self.tcp_stream = Some(stream);
                    },
                    Err(_) => {},
                }
            }
        }
    }

    /// Receive client message
    pub fn client_message(&mut self) -> Option<ClientMessage> {

        if self.inc_msg_size.is_none() {
            self.get_message_size();
        }
        
        self.read_message()

    }

    /// Send server message
    pub fn send_message(&mut self, message : ServerMessage) {

       match message.pack_bytes(&mut self.buffer) {
            Ok(size) => self.tcp_stream.as_mut().unwrap().write_all(&self.buffer[0..size]).unwrap(),
            Err(err) => panic!("send_message err({:?})!", err),
        }

    }

    fn read_message(&mut self) -> Option<ClientMessage> {

        // Try to read message if any
        if let Some(msg_size) = self.inc_msg_size {
            match self.tcp_stream.as_mut().unwrap().read_exact(&mut self.buffer[..msg_size]) {
                Ok(_) => match ClientMessage::from_bytes(&self.buffer[..msg_size]){
                        Ok(msg) => {
                            self.inc_msg_size = None;
                            Some(msg) // Return message
                        },
                        Err(err) => panic!("read_message err({:?})!", err),
                    }
                Err(err) => {
                    match err.kind() {
                        std::io::ErrorKind::WouldBlock => None,
                        _ => panic!("read_message err({:?})!", err),
                    } 
                },
            } 
        } else {
            None
        }
    }

    #[inline]
    fn get_message_size(&mut self) {
        
        self.inc_msg_size = match self.tcp_stream.as_mut().unwrap().read_exact(&mut self.buffer[..MESSAGE_SIZE_TYPE_SIZE]) {
            Ok(_) => Some(ClientMessage::size_from_bytes(&self.buffer[0..MESSAGE_SIZE_TYPE_SIZE].try_into().unwrap()) as usize),
            Err(err) => match err.kind() {
                std::io::ErrorKind::WouldBlock => todo!(),
                _ => panic!("get_message_size err({:?})!", err),
            },
        };        
    }

}