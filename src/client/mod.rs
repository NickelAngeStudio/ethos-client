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

use std::{sync::mpsc::{self, *}, thread::JoinHandle, time::{Duration, Instant}};

use crate::client::{message::*, thread::ClientThread};

#[cfg(test)]
mod tests;

pub mod message;
pub mod thread;
pub mod error;

use debug_print::debug_eprintln;
use error::Error as ClientError;
use ethos_core::net::{ClientMessage, ServerMessage};

/// Reservation size for element of update_vec
const UPDATE_VEC_RESERVE : usize = 100;

/// Reservation size for element of message_vec
const MESSAGE_VEC_RESERVE : usize = 100;

/// Maximum duration while trying to join thread.
const DROP_WAIT_DURATION : Duration = Duration::from_millis(100);

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

    /// Client thread has ended
    Ended,

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

    /// Thread communication channels
    channels : EthosClientChannel,
    
    /// Handle of the thread
    thread_handle : Option<JoinHandle<()>>,

    /// Status of the net client
    status : EthosClientStatus, 

    /// Vector used to hold [EthosClientUpdate] for [EthosClient::update_vec()]
    update_vec : Vec<EthosClientUpdate>,

    /// Vector used to hold [ServerMessage] for [EthosClient::message_vec()]
    message_vec : Vec<ServerMessage>

}

/// Thread communication channels of [EthosClient]
struct EthosClientChannel {
    /// Sender of message to thread
    sdr_ctot : Sender<CtoTMessage>,

    /// Receiver of messages from thread
    rcv_ttoc : Receiver<EthosClientUpdate>,

    /// Receiver of messages from server
    rcv_stoc : Receiver<StoCMessage>,
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

        // Channels
        let (sdr_ctot, rcv_ctot) = mpsc::channel::<CtoTMessage>();
        let (sdr_ttoc, rcv_ttoc) = mpsc::channel::<EthosClientUpdate>();
        let (sdr_stoc, rcv_stoc) = mpsc::channel::<StoCMessage>();
        let channels = EthosClientChannel { sdr_ctot, rcv_ttoc, rcv_stoc };

        // Thread
        let mut ct  = ClientThread::new(rcv_ctot, sdr_ttoc, sdr_stoc);
        let thread_handle = std::thread::spawn(move || { ct.execute(); });

        EthosClient { 
            update_vec, message_vec,
            channels,
            thread_handle : Some(thread_handle),
            status: EthosClientStatus::Disconnected, }

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

                match self.channels.sdr_ctot.send(CtoTMessage::OpenConnection(connect_string)){
                    Ok(_) => Ok(()),
                    Err(_) => {
                        self.status = EthosClientStatus::Disconnected;
                        Err(ClientError::ClientChannelDown)
                    },
                }
                
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

        match self.status {
            EthosClientStatus::Connected => {
                match self.channels.sdr_ctot.send(CtoTMessage::CloseConnection){
                    Ok(_) => Ok(()),
                    Err(_) => Err(ClientError::SendClientMessageFailed),
                }
            },
            _ => Err(ClientError::ClientDisconnected),
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

        match self.channels.rcv_ttoc.try_recv() {
            Ok(msg) => {
                match msg {
                    EthosClientUpdate::Error(error) => self.handle_update_error(error),
                    EthosClientUpdate::StatusChanged(status) =>self.handle_update_status_changed(status),
                }
            },
            Err(_) => None,
        }
        
    }

    /// Handle error of update
    #[inline]
    fn handle_update_error(&mut self, err : ClientError) -> Option<EthosClientUpdate> {


        match &err {    // Set client as disconnecting since connection never happened
            ClientError::ClientDisconnected | ClientError::InvalidConnectionString | 
                ClientError::UnhandledIOError(_) | ClientError::ServerDown => {
                self.status = EthosClientStatus::Disconnected;  // Set status as disconnected

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

        match self.status {
            EthosClientStatus::Connected => {
                match self.channels.sdr_ctot.send(CtoTMessage::SendMessage(message)){
                    Ok(_) => Ok(()),
                    Err(_) => Err(ClientError::SendClientMessageFailed),
                }
            },
            _ => Err(ClientError::ClientDisconnected)
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

        match self.status {
            EthosClientStatus::Connected => {
                // Remove previous messages
                self.message_vec.clear(); 

                'message_vec:
                loop {
                    match self.channels.rcv_stoc.try_recv() {
                        Ok(msg) => match msg {
                            StoCMessage::Message(server_message) => self.message_vec.push(server_message),
                            _ => {},    // Ignore other message
                        },
                        Err(_) => break 'message_vec,
                    }
                }

                Ok(&self.message_vec)

            },
            _ => Err(ClientError::ClientDisconnected),
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

         match self.status {
            EthosClientStatus::Connected => {
                match self.channels.rcv_stoc.try_recv() {
                    Ok(msg) => match msg {
                        StoCMessage::Message(server_message) => Ok(Some(server_message)),
                        _ => Ok(None),
                    },
                    Err(_) => Ok(None),
                }

            },
            _ => Err(ClientError::ClientDisconnected),
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

impl Drop for EthosClient {
    fn drop(&mut self) {

        // Ask client to end thread
        match self.channels.sdr_ctot.send(CtoTMessage::EndClient){
            Ok(_) => {
                let ts = Instant::now();
                'join:
                loop {
                    match self.thread_handle.as_ref() {
                        Some(th) => {
                            if th.is_finished() {   // If thread is finished, take it and end.
                                match self.thread_handle.take(){
                                    Some(th) => {
                                        match th.join(){
                                            Ok(_) => {
                                                debug_eprintln!("Drop : Thread joined!");
                                            },
                                            Err(_) => {
                                                debug_eprintln!("Drop : Thread join failed!");
                                            },
                                        }
                                        
                                    },
                                    None => {
                                        debug_eprintln!("Drop : Couldn't take thread handle!");
                                        break 'join;
                                    },
                                };
                                break 'join;
                            }
                        },
                        None => {
                            debug_eprintln!("Drop : Thread handle lost!");
                            break 'join;   // Should not happens.
                        }
                    }

                    if ts.elapsed() > DROP_WAIT_DURATION { // Join took too long
                        debug_eprintln!("Drop : Thread join timeout!");
                        break 'join;
                    }
                }
            },
            Err(_) => { // Communication to thread lost.
                debug_eprintln!("Couldn't EndClient. Channel closed!");
            },   
        }

    }
}