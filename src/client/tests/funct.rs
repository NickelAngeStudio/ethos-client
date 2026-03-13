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

use std::time::Duration;

use ethos_core::net::{ClientMessage, ServerMessage, TCP_PORT};

use crate::{EthosClient, EthosClientStatus, EthosClientUpdate, client::tests::server::UnitTestServer};

pub const LOOP_WAIT_UPDATE_TIME : Duration = std::time::Duration::from_millis(5000);

/// IP address used for test
pub const IP_V4_TEST : &str = "127.0.0.1";

#[macro_export]
macro_rules! timeout_loop {

    ($duration : expr, $($arg:tt)*) => {

        let timestamp = std::time::Instant::now();
        loop {
            $($arg)*

            if timestamp.elapsed() > $duration {
                panic!("Timeout!");
            }
        }

    };

    ($($arg:tt)*) => {
        timeout_loop!($crate::client::tests::LOOP_WAIT_TIME, $($arg)*);
    };
}

/// Loop that run it's full duration.
#[macro_export]
macro_rules! timeloop {

    ($duration : expr, $($arg:tt)*) => {

        let timestamp = std::time::Instant::now();

        'timeloop:
        loop {
            $($arg)*

            if timestamp.elapsed() > $duration {
                break 'timeloop;
            }
        }

    };

    ($($arg:tt)*) => {
        timeloop!($crate::client::tests::LOOP_WAIT_TIME, $($arg)*);
    };
}


pub(super) fn wait_update(client : &mut EthosClient) -> EthosClientUpdate {

    timeout_loop!{
        match client.update() {
            Some(update) => return update,
            None => {},
        }
    };
    
}

pub(super) fn wait_server_message(client : &mut EthosClient) -> ServerMessage {

    timeout_loop!{
        match client.message() {
            Ok(opt) => match opt {
                Some(message) => return message,
                None => {},
            },
            Err(err) => panic!("client.message shouldn't err({:?})", err),
        }
    };
    
}

pub(super) fn wait_client_message(server : &mut UnitTestServer) -> ClientMessage {

    timeout_loop!{
        match server.client_message() {
            Some(msg) => return msg,
            None => {},
        }
    };
    
}


pub(super) fn wait_disconnected(client : &mut EthosClient) {
     wait_update_message(client, EthosClientUpdate::StatusChanged(EthosClientStatus::Disconnected));
}

pub(super) fn wait_update_message(client : &mut EthosClient, msg : EthosClientUpdate) {

    timeout_loop!{ LOOP_WAIT_UPDATE_TIME,
        if msg == wait_update(client) {
            return;
        }
    }

}

pub(super) fn connect_string(port_minus : u16) -> String {
    format!("{}:{}", IP_V4_TEST, TCP_PORT - port_minus)
}

pub(super) fn prepare_server(port_minus : u16) -> UnitTestServer {
    let connect_string = connect_string(port_minus);
    
    let mut server = UnitTestServer::new();
    server.start(connect_string);
    server
}

pub(super) fn prepare_client(port_minus : u16) -> EthosClient {
    let connect_string = connect_string(port_minus);

    let mut client = EthosClient::new();
    client.connect(connect_string).unwrap();
    wait_update_message(&mut client, EthosClientUpdate::StatusChanged(EthosClientStatus::Connected));

    client
}

pub(super) fn prepare_client_no_wait(port_minus : u16) -> EthosClient {
    let connect_string = connect_string(port_minus);

    let mut client = EthosClient::new();
    client.connect(connect_string).unwrap();

    client
}

pub(super) fn prepare_server_client(port_minus : u16) -> (UnitTestServer, EthosClient) {

    let mut server = prepare_server(port_minus);
    let client = prepare_client(port_minus);
    server.accept();
    (server, client)

} 

pub(super) fn prepare_server_client_no_wait(port_minus : u16) -> (UnitTestServer, EthosClient) {

    let mut server = prepare_server(port_minus);
    let client = prepare_client_no_wait(port_minus);
    server.accept();

    (server, client)

} 

