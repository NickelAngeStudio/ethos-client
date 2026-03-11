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

use std::{net::TcpStream, sync::mpsc::{Receiver, Sender}};

use debug_print::debug_eprintln;
use ethos_core::net::SERVER_MSG_BUFFER_SIZE;
use crate::Error as ClientError;

use crate::{EthosNetClient, EthosNetClientUpdate, client::message::{CtoTMessage, StoCMessage}};

/// Client thread parameters
pub(crate) struct ClientThread {
    pub(crate)  connect_string : String, 
    pub(crate)  rcv_ctot : Receiver<CtoTMessage>, 
    pub(crate)  sdr_ttoc : Sender<EthosNetClientUpdate>, 
    pub(crate)  sdr_stoc : Sender<StoCMessage>
}

impl EthosNetClient {

    
    pub(super) fn handle_client_thread(mut ct : ClientThread) {

        // Put buffer on heap
        let mut buffer = Vec::<u8>::new();
        buffer.resize(SERVER_MSG_BUFFER_SIZE, 0);

        // Return value
        let mut return_value : usize = 0;

        match TcpStream::connect(ct.connect_string.clone()){
            Ok(mut stream) => {
                stream.set_nonblocking(true).unwrap();

                /*
                sdr_ttoc.send(TtoCMessage::Connected).unwrap();

                loop {
                    Self::handle_client_message(&mut stream, &mut return_value, &mut buffer, &mut rcv_ctot);

                    if return_value == 0 {
                        Self::handle_server_message(&mut stream, &mut return_value, &mut buffer, &mut sdr_ttoc);
                    } else {
                        break;
                    }
                }

                // Shutdown stream
                stream.shutdown(std::net::Shutdown::Both).unwrap();
                */

            },
            Err(err) => { 
                match err.kind() {
                    // Invalid connect string
                    std::io::ErrorKind::InvalidInput => {
                        debug_eprintln!("handle_client_thread (ClientError::InvalidConnectionString), err({:?})",err);
                        ct.sdr_ttoc.send(EthosNetClientUpdate::Error(ClientError::InvalidConnectionString)).unwrap();
                    },
                    // Server is down / busy
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::ConnectionRefused | 
                    std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::HostUnreachable | std::io::ErrorKind::NetworkUnreachable | 
                    std::io::ErrorKind::ConnectionAborted  | std::io::ErrorKind::NotConnected => {
                        debug_eprintln!("handle_client_thread (ClientError::ServerDown), err({:?})",err);
                        ct.sdr_ttoc.send(EthosNetClientUpdate::Error(ClientError::ServerDown)).unwrap();
                    }
                    // Other IO error
                    _ => {
                        debug_eprintln!("handle_client_thread (ClientError::UnhandledIOError), err({:?})",err);
                        ct.sdr_ttoc.send(EthosNetClientUpdate::Error(ClientError::UnhandledIOError(err.kind()))).unwrap();
                    },
                }
                
            },
        }

    }
    

}