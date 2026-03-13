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

use ethos_core::net::Error as CoreError;

/// Possible [EthosClient](crate::EthosClient) error.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Happens when trying to do action while client is disconnected
    ClientDisconnected,

    /// Invalid connection string given.
    InvalidConnectionString,

    /// Client is already connected
    ClientAlreadyConnected,

    /// Sending a client message failed. Usually mean channel is closed.
    SendClientMessageFailed,

    /// Server is down
    ServerDown,

    /// Unhandled IO error
    UnhandledIOError(std::io::ErrorKind),

    /// ClientMessage size is too large
    ClientMessageTooLarge,

    /// Error occurred when packing bytes
    ClientMessagePackError(CoreError),

    /// Failed to send message to server
    ClientMessageSendError(std::io::ErrorKind),

    /// Error occurred while reading message from server
    ServerMessageReadError(std::io::ErrorKind),

    /// Error occurred when retrieving ServerMessage from bytes
    ServerMessageFromBytesError(CoreError),

    /// Happens when client channels are down. [EthosClient] must be dropped and recreated.
    ClientChannelDown,

}