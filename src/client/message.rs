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

use ethos_core::net::{ClientMessage, ServerMessage};

use crate::{Error as ClientError, EthosClientStatus};


/// Client to thread message
pub enum CtoTMessage {

    /// Open a connection with connect_string
    OpenConnection(String),

    /// Client message to send to the server
    SendMessage(ClientMessage),

    /// Tell client to close connection
    CloseConnection,

    /// Tell the client to end the client thread
    EndClient,

}

/// Update sent to [EthosClient](crate::EthosClient) from communication thread.
/// 
/// Update should be retrieved via [EthosClient::update()](crate::EthosClient::update) in
/// the main loop of the software connecting to the server.
/// ```no_run
/// // -- snip --
/// 'main:
/// loop {
///     // -- snip --
///     'client_update:
///     loop{
///         match client.update() {
///            Some(update) => {}, // -- Handle update here --
///            None => break 'client_update,
///         }
///     }
///     // -- snip --
/// }
/// ```
#[derive(Debug, PartialEq)]
pub enum EthosClientUpdate {

    /// An error occurred during the thread execution.
    Error(ClientError),

    /// The client status changed. Client is disconnected only when StatusChanged([`EthosClientStatus::Disconnected`](crate::EthosClientStatus::Disconnected)) update is given.
    StatusChanged(EthosClientStatus),

}

/// Wrapper of message coming from the remote server.
pub(crate) enum StoCMessage {
    Message(ServerMessage)
}