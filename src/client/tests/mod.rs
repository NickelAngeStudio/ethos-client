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

//! EthosNetClient unit tests
//! V1 : Create a new EthosNetClient
//! V2 : Incorrect connection string should give InvalidConnectionString 
//! V3 : Correct connection string should connect
//! V4 : Incorrect connection then correct connection should work.
//! V5 : Trying to connect twice give ClientAlreadyConnected
//! V6 : Close connection give ClientDisconnected when disconnected
//! V7 : Close connection give Ok(()) when connected
//! V8 : send_message give ClientDisconnected when disconnected
//! V9 : send_message send a message successfully
//! V10 : server_message give ClientDisconnected when disconnected
//! V11 : server_message receive None when no message
//! V11 : server_message receive Some(ServerMessage)
//! V13 : update join handle and clear channels.
//! V14 : Handle server connection drop
//! V15 : Trying to connect when server down should return ServerDown.
//! V16 : ServerDown then server up connection should work.
//! V17 : ClientMessageTooLarge
use std::time::Duration;

use crate::{EthosClient, EthosClientStatus, EthosClientUpdate, client::{self, tests::funct::{connect_string, prepare_client, prepare_server, prepare_server_client, wait_disconnected, wait_update, wait_update_message}}, timeloop, timeout_loop};
use crate::client::error::Error as ClientError;

mod funct;
mod server;

/// Maximum loop wait time.
pub const LOOP_WAIT_TIME : Duration = std::time::Duration::from_millis(5000);

#[test]
fn v1_create_client(){
    // V1 : Create a new EthosNetClient
    let _client = EthosClient::new();

}
#[test]
fn v2_connect_invalid(){
    // V2 : Incorrect connection string should give InvalidConnectionString 
    let mut client = EthosClient::new();
    let _server = prepare_server(2);

    client.connect("nothing".to_string()).unwrap();
    wait_update_message(&mut client, EthosClientUpdate::Error(ClientError::InvalidConnectionString));

    wait_disconnected(&mut client);
}

#[test]
fn v15_server_down(){
    // V15 : Trying to connect when server down should return ServerDown.
    let mut client = prepare_client(15);

    wait_update_message(&mut client, EthosClientUpdate::Error(ClientError::ServerDown));

    wait_disconnected(&mut client);

}

#[test]
fn v3_connect_valid(){

    // V3 : Correct connection string should connect
    let (_server, mut client) = prepare_server_client(3);

    wait_update_message(&mut client, EthosClientUpdate::StatusChanged(EthosClientStatus::Connected));

    let duration = Duration::from_millis(500);
    timeloop!{ duration,
        assert_eq!(client.status(), EthosClientStatus::Connected);
    }

}

#[test]
fn v4_connect_invalid_then_valid(){

    // V4 : Incorrect connection then correct connection should work.
    let mut client = EthosClient::new();
    let _server = prepare_server(4);

    // Invalid
    client.connect("nothing".to_string()).unwrap();
    wait_update_message(&mut client, EthosClientUpdate::Error(ClientError::InvalidConnectionString));
    wait_disconnected(&mut client);

    // Valid
    client.connect(connect_string(4)).unwrap();
    wait_update_message(&mut client, EthosClientUpdate::StatusChanged(EthosClientStatus::Connected));

}

#[test]
fn v16_server_down_up() {
    // V16 : ServerDown then server up connection should work.
    let mut client = prepare_client(16);

    wait_update_message(&mut client, EthosClientUpdate::Error(ClientError::ServerDown));
    wait_disconnected(&mut client);

    // Server goes up
    let _server = prepare_server(16);
    client.connect(connect_string(16)).unwrap();
    wait_update_message(&mut client, EthosClientUpdate::StatusChanged(EthosClientStatus::Connected));
}


#[test]
fn v5_connect_already(){
    // V5 : Trying to connect twice give ClientAlreadyConnected
    let (_server, mut client) = prepare_server_client(5);

    match client.connect(connect_string(5)) {
        Ok(_) => panic!("v5 should not be Ok()!"),
        Err(err) => assert_eq!(err, ClientError::ClientAlreadyConnected),
    }

}

#[test]
fn v6_close_connection_disc(){
    // V6 : Close connection give ClientDisconnected when disconnected
    let mut client = EthosClient::new();

    match client.close() {
        Ok(_) => panic!("v6 shouldn't be Ok()!"),
        Err(err) => assert_eq!(err, ClientError::ClientDisconnected),
    }
}

#[test]
fn v7_close_connection(){
    // V7 : Close connection give Ok(()) when connected
    todo!()
}

#[test]
fn v8_send_message_disc(){
    // V8 : send_message give ClientDisconnected when disconnected
    todo!()
}

#[test]
fn v9_send_message(){
    // V9 : send_message send a message successfully
    todo!()
}

#[test]
fn v10_server_message_disc(){
    // V10 : server_message give ClientDisconnected when disconnected
    todo!()
}

#[test]
fn v11_server_message_none(){
    // V11 : server_message receive None when no message
    todo!()
}

#[test]
fn v12_server_message_some(){
    // V12 : server_message receive Some(ServerMessage)
    todo!()
}

#[test]
fn v13_update_join_handle(){
    // V13 : update join handle.
    todo!()
}

#[test]
fn v14_handle_connection_drop() {
    // V14 : Handle server connection drop

}

#[test]
fn v17_client_message_too_large() {
    // V17 : ClientMessageTooLarge

}