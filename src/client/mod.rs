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

use std::{sync::mpsc::{self, *}, thread::JoinHandle};

use crate::client::{message::*, thread::ClientThread};

#[cfg(test)]
mod tests;

pub mod message;
pub mod thread;
pub mod error;

use error::Error as ClientError;
use ethos_core::net::{ClientMessage, ServerMessage};

/// Current status of [EthosNetClient].
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EthosNetClientStatus {
    /// Client is currently disconnected.
    Disconnected,

    /// Client is currently trying to connect
    Connecting,

    /// Client is currently connected.
    Connected,

    /// Client is currently disconnecting
    Disconnecting,

}

/// Client that communicate with ethos server.
pub struct EthosNetClient {
    /// Sender of message to thread
    sdr_ctot : Option<Sender<CtoTMessage>>,

    /// Receiver of messages from thread
    rcv_ttoc : Option<Receiver<EthosNetClientUpdate>>,

    /// Receiver of messages from server
    rcv_stoc : Option<Receiver<StoCMessage>>,

    /// Handle of the thread
    thread_handle : Option<JoinHandle<()>>,

    /// Status of the net client
    status : EthosNetClientStatus,
}

impl EthosNetClient {
    /// Create a new [Disconnected](EthosNetClientStatus::Disconnected) EthosNetClient.
    /// 
    /// # Returns
    /// - A new EthosNetClient.
    pub fn new() -> EthosNetClient {
        EthosNetClient { sdr_ctot: None, rcv_ttoc: None, 
            thread_handle: None, status: EthosNetClientStatus::Disconnected, rcv_stoc: None }
    }

    /// Connect the client to server via a connection string.
    /// 
    /// Connection string should be `host:port`. See [std::net::TcpStream] for more.
    /// 
    /// The result of this operation will be transmitted via [`EthosNetClientUpdate`].
    /// 
    /// # Returns 
    /// - Result
    ///     - Ok(()) if connect query is successfull. Result is sent via [`EthosNetClientUpdate`].
    ///     - Err([`ClientAlreadyConnected`](crate::Error::ClientAlreadyConnected)) if client is already connected.
    pub fn connect(&mut self, connect_string : String) -> Result<(), ClientError> {

        if self.thread_handle.is_none() {
            let (sdr_ctot, rcv_ctot) = mpsc::channel::<CtoTMessage>();
            let (sdr_ttoc, rcv_ttoc) = mpsc::channel::<EthosNetClientUpdate>();
            let (sdr_stoc, rcv_stoc) = mpsc::channel::<StoCMessage>();

            self.sdr_ctot = Some(sdr_ctot);
            self.rcv_ttoc = Some(rcv_ttoc);
            self.rcv_stoc = Some(rcv_stoc);

            // Start client thread
            self.thread_handle = Some(std::thread::spawn(move || {
                Self::handle_client_thread(ClientThread { connect_string, rcv_ctot, sdr_ttoc, sdr_stoc });
            }));

            Ok(())
        } else {
            Err(ClientError::ClientAlreadyConnected)
        }
        

    }

    /// Close the connection to the server.
    /// 
    /// # Returns
    /// - Result
    ///     - Ok(())  if disconnect query is successfull. Result is sent via [`EthosNetClientUpdate`].
    ///     - Err([ClientDisconnected](crate::Error::ClientDisconnected)) if client already disconnected.
    pub fn close(&mut self) -> Result<(), ClientError> {

        todo!()

    }

    /// Receive [`EthosNetClientUpdate`] from the client thread.
    /// 
    /// [`EthosNetClient`] is non-blocking and action like `connect()` and `close()`
    /// result are sent via [`EthosNetClientUpdate`] thus `update()` must be handle in main loop.
    /// 
    /// 
    /// # Returns
    /// - [Result]
    ///     - Some([`EthosNetClientUpdate`]) if update found.
    ///     - None if no update.
    pub fn update(&mut self) -> Option<EthosNetClientUpdate> {
        
        match self.rcv_ttoc.as_mut() {
            Some(rcv) => match rcv.try_recv() {
                Ok(msg) => Some(msg),
                Err(_) => None,
            },
            None => None,
        }
        
    }


    /// Send a [ClientMessage] message to remote server.
    /// 
    /// # Returns
    /// - Result
    ///     - Ok(()) if message is sent.
    ///     - Err([ClientDisconnected](crate::Error::ClientDisconnected)) if client is disconnected.
    ///     - Err([SendClientMessageFailed](crate::Error::SendClientMessageFailed)) if send channel is closed.
    pub fn send_message(&mut self, message : ClientMessage) -> Result<(), ClientError> {
        
        match self.sdr_ctot.as_mut() {
            Some(sender) => match sender.send(CtoTMessage::Message(message)){
                Ok(_) => Ok(()),
                Err(_) => Err(ClientError::SendClientMessageFailed),
            },
            None => Err(ClientError::ClientDisconnected),
        }

    }


    /// Receive message from server.
    /// 
    /// # Returns
    /// - Result
    ///     - Ok(Some([ServerMessage])) if message found.
    ///     - Ok(None) if no message found.
    ///     - Err([ClientDisconnected](crate::Error::ClientDisconnected)) if client is disconnected.
    pub fn server_message(&mut self) -> Result<Option<ServerMessage>, ClientError> {

        match self.rcv_stoc.as_mut() {
            Some(rcv) => match rcv.try_recv() {
                Ok(msg) => match msg {
                    StoCMessage::Message(server_message) => Ok(Some(server_message)),
                    _ => Ok(None),
                },
                Err(_) => Ok(None),
            },
            None => Err(ClientError::ClientDisconnected),
        }
        
    }


    /// Get current client status.
    /// 
    /// # Returns
    /// - [EthosNetClientStatus] of client.
    pub fn status(&self) -> EthosNetClientStatus {
        self.status
    }





}