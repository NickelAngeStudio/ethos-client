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

/// Reservation size for element of update_vec
const UPDATE_VEC_RESERVE : usize = 100;

/// Reservation size for element of message_vec
const MESSAGE_VEC_RESERVE : usize = 100;

/// Current status of [EthosClient].
/// 
/// Status of client is updated using [EthosClient::update].
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
/// 
/// # Usage
/// ```no_run
/// use ethos_client::*;
/// // -- snip --
/// let client = EthosClient::new();
/// client.connect(format!("127.0.0.1:{}", TCP_PORT)).unwrap();
/// // -- snip --
/// 'main:
/// loop {
///     // -- snip --
///     for update in client.update_vec() {
///         // -- handle EthosClientUpdate --
///     }
/// 
///     match client.message_vec(){ // Incoming message(s) from server
///         Ok(message_vec) => for message in message_vec {
///             // -- handle ServerMessage --
///         },
///         Err(_) => {},   // -- client is disconnected --
///     }
/// }
/// 
/// ```
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

    /// Vector used to hold [EthosClientUpdate] for [EthosClient::update_vec()]
    update_vec : Vec<EthosClientUpdate>,

    /// Vector used to hold [ServerMessage] for [EthosClient::message_vec()]
    message_vec : Vec<ServerMessage>
}

impl EthosClient {
    /// Create a new [`Disconnected`](EthosClientStatus::Disconnected) [`EthosClient`].
    /// 
    /// # Returns
    /// - New [`EthosClient`]
    pub fn new() -> EthosClient {
        let (mut update_vec, mut message_vec) = (Vec::<EthosClientUpdate>::new(), Vec::<ServerMessage>::new());
        update_vec.reserve(UPDATE_VEC_RESERVE);
        message_vec.reserve(MESSAGE_VEC_RESERVE);

        EthosClient { sdr_ctot: None, rcv_ttoc: None, 
            thread_handle: None, status: EthosClientStatus::Disconnected, rcv_stoc: None,
            update_vec, message_vec}
    }

    /// Connect the client to server via a provided connection string.
    /// 
    /// Connection string should be `host:port`. See [`TcpStream`](std::net::TcpStream) for more information.
    /// 
    /// The result of this operation will be transmitted via [`EthosClientUpdate`].
    /// 
    /// # Returns 
    /// - [`Result`]
    ///     - Ok(()) if connect query is successful. Result is sent via [`EthosClientUpdate`].
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
    /// - [`Result`]
    ///     - Ok(())  if disconnect query is successful. Result is sent via [`EthosClientUpdate`].
    ///     - Err([`ClientDisconnected`](crate::Error::ClientDisconnected)) if client already disconnected.
    pub fn close(&mut self) -> Result<(), ClientError> {

        match self.sdr_ctot.as_mut() {
            Some(sender) => match sender.send(CtoTMessage::CloseConnection){
                Ok(_) => Ok(()),
                Err(_) => Err(ClientError::SendClientMessageFailed),
            },
            None => Err(ClientError::ClientDisconnected),
        }

    }

    /// Receive a reference of all current [`EthosClientUpdate`] in a vector.
    /// 
    /// # Returns
    /// - Reference to vector of [`EthosClientUpdate`].
    pub fn update_vec(&mut self) -> &Vec<EthosClientUpdate> {
       // Remove previous update
       self.update_vec.clear(); 
       

        'update_vec:
        loop {
            match self.update() {
                Some(update) => self.update_vec.push(update),
                None => break 'update_vec,
            }
        }

        &self.update_vec
    }

    /// Receive a [`EthosClientUpdate`] from the client thread.
    /// 
    /// [`EthosClient`] is non-blocking and action like `connect()` and `close()`
    /// result are sent via [`EthosClientUpdate`].
    /// 
    /// # Returns
    /// - [`Result`]
    ///     - Some([`EthosClientUpdate`]) if update found.
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


        match &err {    // Set client as disconnecting since connection never happened
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


    /// Send a [`ClientMessage`] to remote ethos server.
    /// 
    /// # Returns
    /// - [`Result`]
    ///     - Ok() if message is sent.
    ///     - Err([`ClientDisconnected`](crate::Error::ClientDisconnected)) if client is disconnected.
    ///     - Err([`SendClientMessageFailed`](crate::Error::SendClientMessageFailed)) if send channel is closed.
    pub fn send(&mut self, message : ClientMessage) -> Result<(), ClientError> {
        
        match self.sdr_ctot.as_mut() {
            Some(sender) => match sender.send(CtoTMessage::SendMessage(message)){
                Ok(_) => Ok(()),
                Err(_) => Err(ClientError::SendClientMessageFailed),
            },
            None => Err(ClientError::ClientDisconnected),
        }

    }

    /// Receive a reference to all current [`ServerMessage`] in a vector.
    /// 
    /// # Returns
    /// - [`Result`]
    ///     - Ok() with reference to vector of [`ServerMessage`].
    ///     - Err([`ClientDisconnected`](crate::Error::ClientDisconnected)) if client is disconnected.
    #[allow(unreachable_patterns)]
    pub fn message_vec(&mut self) -> Result<&Vec<ServerMessage>, ClientError> {

       match self.rcv_stoc.as_mut() {
            Some(rcv) => {
                // Remove previous messages
                self.message_vec.clear(); 

                'message_vec:
                loop {
                    match rcv.try_recv() {
                        Ok(msg) => match msg {
                            StoCMessage::Message(server_message) => self.message_vec.push(server_message),
                            _ => {},    // Ignore other message
                        },
                        Err(_) => break 'message_vec,
                    }
                }

                Ok(&self.message_vec)


            },
            None => Err(ClientError::ClientDisconnected),
        }

    }


    /// Receive a [`ServerMessage`] from remote ethos server.
    /// 
    /// # Returns
    /// - [`Result`]
    ///     - Ok(Some([`ServerMessage`])) if message found.
    ///     - Ok(None) if no message found.
    ///     - Err([ClientDisconnected](crate::Error::ClientDisconnected)) if client is disconnected.
    #[allow(unreachable_patterns)]
    pub fn message(&mut self) -> Result<Option<ServerMessage>, ClientError> {

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


    /// Get current [`EthosClientStatus`].
    /// 
    /// # Returns
    /// - [`EthosClientStatus`] of client.
    pub fn status(&self) -> EthosClientStatus {
        self.status
    }


}