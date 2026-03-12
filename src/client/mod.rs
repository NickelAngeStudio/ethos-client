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
pub enum EthosClientStatus {
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
pub struct EthosClient {
    /// Sender of message to thread
    sdr_ctot : Option<Sender<CtoTMessage>>,

    /// Receiver of messages from thread
    rcv_ttoc : Option<Receiver<EthosClientUpdate>>,

    /// Receiver of messages from server
    rcv_stoc : Option<Receiver<StoCMessage>>,

    /// Handle of the thread
    thread_handle : Option<JoinHandle<()>>,

    /// Status of the net client
    status : EthosClientStatus, 
}

impl EthosClient {
    /// Create a new [Disconnected](EthosNetClientStatus::Disconnected) EthosNetClient.
    /// 
    /// # Returns
    /// - A new EthosNetClient.
    pub fn new() -> EthosClient {
        EthosClient { sdr_ctot: None, rcv_ttoc: None, 
            thread_handle: None, status: EthosClientStatus::Disconnected, rcv_stoc: None }
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

        match self.status {
            EthosClientStatus::Disconnected => { // Only connect if client is disconnected
                self.status = EthosClientStatus::Connecting; // Change status to connecting

                let (sdr_ctot, rcv_ctot) = mpsc::channel::<CtoTMessage>();
                let (sdr_ttoc, rcv_ttoc) = mpsc::channel::<EthosClientUpdate>();
                let (sdr_stoc, rcv_stoc) = mpsc::channel::<StoCMessage>();

                self.sdr_ctot = Some(sdr_ctot);
                self.rcv_ttoc = Some(rcv_ttoc);
                self.rcv_stoc = Some(rcv_stoc);

                // Start client thread
                self.thread_handle = Some(std::thread::spawn(move || {
                    Self::handle_client_thread(ClientThread { connect_string, rcv_ctot, sdr_ttoc, sdr_stoc, status: EthosClientStatus::Connecting, inc_size: None });
                }));

                Ok(())
            },
            _ => Err(ClientError::ClientAlreadyConnected),
        }


    }

    /// Close the connection to the server.
    /// 
    /// # Returns
    /// - Result
    ///     - Ok(())  if disconnect query is successfull. Result is sent via [`EthosNetClientUpdate`].
    ///     - Err([ClientDisconnected](crate::Error::ClientDisconnected)) if client already disconnected.
    pub fn close(&mut self) -> Result<(), ClientError> {

        match self.sdr_ctot.as_mut() {
            Some(sender) => match sender.send(CtoTMessage::CloseConnection){
                Ok(_) => Ok(()),
                Err(_) => Err(ClientError::SendClientMessageFailed),
            },
            None => Err(ClientError::ClientDisconnected),
        }

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
    pub fn update(&mut self) -> Option<EthosClientUpdate> {

        // If Disconnecting, final step is to disconnect and join thread
        if self.status == EthosClientStatus::Disconnecting {
            self.handle_update_disconnecting()
        } else {
             // Get message
            match self.rcv_ttoc.as_mut() {
                Some(rcv) => match rcv.try_recv() {
                    Ok(msg) => {
                        match msg {
                            EthosClientUpdate::Error(error) => self.handle_update_error(error),
                            EthosClientUpdate::StatusChanged(status) =>self.handle_update_status_changed(status),
                        }
                    },
                    Err(_) => None,
                },
                None => None,
            }
        }
        
    }

    /// Handle final non-blocking step to disconnect client.
    #[inline]
    fn handle_update_disconnecting(&mut self) -> Option<EthosClientUpdate> {

        // Is thread ready to join?
        if self.thread_handle.as_ref().unwrap().is_finished() {
            self.thread_handle.take().unwrap().join().unwrap();  // Join thread

            // Close channels
            self.sdr_ctot.take();
            self.rcv_stoc.take();
            self.rcv_ttoc.take();

            // Set status as disconnected.
            self.status = EthosClientStatus::Disconnected;
            Some(EthosClientUpdate::StatusChanged(EthosClientStatus::Disconnected))
        } else {
            None
        }


    }

    /// Handle error of update
    #[inline]
    fn handle_update_error(&mut self, err : ClientError) -> Option<EthosClientUpdate> {


        match &err {    // Set client as disconnecting
            ClientError::ClientDisconnected | ClientError::InvalidConnectionString | 
                ClientError::UnhandledIOError(_) | ClientError::ServerDown => {
                self.status = EthosClientStatus::Disconnecting;  // Set status as disconnecting

            }
            _ => {}
        }

        Some(EthosClientUpdate::Error(err))
    }

    /// Handle status changed of update.
    #[inline]
    fn handle_update_status_changed(&mut self, new_status : EthosClientStatus) -> Option<EthosClientUpdate> {

        self.status = new_status;
        Some(EthosClientUpdate::StatusChanged(new_status)) 

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
            Some(sender) => match sender.send(CtoTMessage::SendMessage(message)){
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
    pub fn status(&self) -> EthosClientStatus {
        self.status
    }





}