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

use std::{sync::mpsc::*, thread::JoinHandle};

use crate::client::message::*;

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
    /// Connection string of client
    connect_string : Option<String>,

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
        EthosNetClient { connect_string : None, sdr_ctot: None, rcv_ttoc: None, 
            thread_handle: None, status: EthosNetClientStatus::Disconnected, rcv_stoc: None }
    }

    /// Connect the client to server via a connection string.
    /// 
    /// Connection string should be `host:port`.
    /// 
    /// The result of this operation will be transmitted via [`EthosNetClientUpdate`].
    /// 
    /// # Returns 
    /// - Result
    ///     - Ok(()) if connect query is successfull. Result is sent via [`EthosNetClientUpdate`].
    ///     - Err([`ClientAlreadyConnected`](crate::Error::ClientAlreadyConnected)) if client is already connected.
    pub fn connect(&mut self, connect_string : String) -> Result<(), ClientError> {
        todo!()
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
    /// [`EthosNetClient`] is non-blocking and [`EthosNetClientUpdate`] must be handle in main loop.
    /// 
    /// ```no_run
    /// // --snip--
    /// 'main:
    /// loop {
    ///     'client_update:
    ///     loop {
    ///         match client.update() {
    ///             Some(update) => { 
    ///                 // --snip--
    ///             },
    ///             None => break 'client_update,
    ///         }
    ///     }
    ///     // --snip--
    /// 
    /// }
    /// 
    /// 
    /// ```
    /// 
    /// # Returns
    /// - [Result]
    ///     - Some([`EthosNetClientUpdate`]) if update found.
    ///     - None if no update.
    /// 
    pub fn update(&mut self) -> Option<EthosNetClientUpdate> {
        
        match self.rcv_ttoc.as_mut() {
            Some(rcv) => match rcv.try_recv() {
                Ok(msg) => Some(msg),
                Err(_) => None,
            },
            None => None,
        }
        
    }

    /// Send message to server
    /// 
    /// # Returns
    /// - Result
    ///     - Ok(()) if message is sent.
    ///     - Err([ClientDisconnected](crate::Error::ClientDisconnected)) if client is disconnected.
    pub fn send_message(&mut self, message : ClientMessage) -> Result<(), ClientError> {
        
        todo!()

    }


    /// Receive message from server.
    /// 
    /// # Returns
    /// - Result
    ///     - Ok(Some([ServerMessage])) if message found.
    ///     - Ok(None) if no message found.
    ///     - Err([ClientDisconnected](crate::Error::ClientDisconnected)) if client is disconnected.
    pub fn server_message(&mut self) -> Result<Option<ServerMessage>, ClientError> {
        
        // No need to handle_thread_message since message are stored locally.

        match self.rcv_stoc.as_mut() {
            Some(_) => todo!(),
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