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
//! V13 : Close Connection update join handle and clear channels.
//! V14 : Handle server connection drop
//! V15 : Trying to connect when server down should return ServerDown.
//! V16 : ServerDown then server up connection should work.
//! V17 : ClientMessageTooLarge
//! V18 : update_vec gives update in a convenient vector.
//! V19 : message_vec gives server message in a convenient vector.
use std::{thread, time::Duration, u16};

use ethos_core::net::{CLIENT_MSG_MAX_SIZE, ClientMessage, ClientPayload, ServerMessage, ServerPayload};

use crate::{EthosClient, EthosClientStatus, EthosClientUpdate, client::tests::funct::{connect_string, prepare_client_no_wait, prepare_server, prepare_server_client, prepare_server_client_no_wait, wait_client_message, wait_server_message, wait_update_message}, timeloop, timeout_loop};
use crate::client::error::Error as ClientError;

mod funct;
mod server;

/// Maximum loop wait time.
pub const LOOP_WAIT_TIME : Duration = std::time::Duration::from_millis(1000);

/// Client payload used for control
const CONTROL_CLIENT_PAYLOAD : ClientPayload = ethos_core::net::ClientPayload::Test { p16: u16::MAX / 4, p32: u32::MAX / 4 };

/// Server payload used for control
const CONTROL_SERVER_PAYLOAD : ServerPayload = ethos_core::net::ServerPayload::Test { p16: u16::MAX / 16, p32: u32::MAX / 16 };
const CONTROL_SERVER_TIMESTAMP : u64 = u64::MAX / 256;

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

}

#[test]
fn v15_server_down(){
    // V15 : Trying to connect when server down should return ServerDown.
    let mut client = prepare_client_no_wait(15);

    wait_update_message(&mut client, EthosClientUpdate::Error(ClientError::ServerDown));

}

#[test]
fn v3_connect_valid(){

    // V3 : Correct connection string should connect
    let (_server, client) = prepare_server_client(3);

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

    // Valid
    client.connect(connect_string(4)).unwrap();
    wait_update_message(&mut client, EthosClientUpdate::StatusChanged(EthosClientStatus::Connected));

}

#[test]
fn v16_server_down_up() {
    // V16 : ServerDown then server up connection should work.
    let mut client = prepare_client_no_wait(16);

    wait_update_message(&mut client, EthosClientUpdate::Error(ClientError::ServerDown));

    // Server goes up
    let mut server = prepare_server(16);
    client.connect(connect_string(16)).unwrap();
    server.accept();
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
    // V7 : Close connection give Ok(()) when connected and send correct update
    let (_server, mut client) = prepare_server_client(7);

    match client.close() {
        Ok(_) => {
            wait_update_message(&mut client, EthosClientUpdate::StatusChanged(EthosClientStatus::Disconnected));
            assert_eq!(client.status(), EthosClientStatus::Disconnected);
        },
        Err(err) => panic!("Close shouldn't err ({:?})", err),
    }
}

#[test]
fn v8_send_message_disc(){
    // V8 : send_message give ClientDisconnected when disconnected
    let mut client = EthosClient::new();

    match client.send(ClientMessage::new(CONTROL_CLIENT_PAYLOAD)) {
        Ok(_) => panic!("v8 shouldn't be Ok()!"),
        Err(err) => assert_eq!(err, ClientError::ClientDisconnected),
    }
    
}

#[test]
fn v9_send_message(){
    // V9 : send_message send a message successfully
    let (mut server, mut client) = prepare_server_client(9);

    client.send(ClientMessage::new(CONTROL_CLIENT_PAYLOAD)).unwrap();

    assert_eq!(wait_client_message(&mut server), ClientMessage::new(CONTROL_CLIENT_PAYLOAD));
}

#[test]
fn v10_server_message_disc(){
    // V10 : server_message give ClientDisconnected when disconnected
    let mut client = EthosClient::new();

    match client.message() {
        Ok(_) => panic!("v10 shouldn't be Ok()!"),
        Err(err) => assert_eq!(err, ClientError::ClientDisconnected),
    }
}

#[test]
fn v11_server_message_none(){
    // V11 : server_message receive None when no message
    let (_server, mut client) = prepare_server_client(11);

    timeloop!{
        // Read messages until None
        match client.message() {
            Ok(msg) => match msg {
                Some(_) => {},
                None => break,
            },
            Err(err) => panic!("V11 shouldn't panic! Err={:?}", err),
        }
    }
    assert!(client.message().unwrap().is_none());
}

#[test]
fn v12_server_message_some(){
    // V12 : server_message receive Some(ServerMessage)
    let (mut server, mut client) = prepare_server_client(12);

    let loop_max : usize = 100;
    let mut loop_index : usize = 0;
    timeout_loop!{
        server.send_message(ServerMessage::new(CONTROL_SERVER_TIMESTAMP, CONTROL_SERVER_PAYLOAD));
        assert_eq!(wait_server_message(&mut client),ServerMessage::new(CONTROL_SERVER_TIMESTAMP, CONTROL_SERVER_PAYLOAD));

        loop_index += 1;

        if loop_index >= loop_max {
            break;
        }
    }
    

}

#[test]
fn v13_update_join_handle(){
    // V13 : Close Connection update join handle and clear channels.
    let (_server, mut client) = prepare_server_client(13);

    match client.close() {
        Ok(_) => {
            wait_update_message(&mut client, EthosClientUpdate::StatusChanged(EthosClientStatus::Disconnected));
            assert_eq!(client.status(), EthosClientStatus::Disconnected);
        },
        Err(err) => panic!("Close shouldn't err ({:?})", err),
    }
}

#[test]
fn v14_handle_connection_drop() {
    // V14 : Handle server connection drop
    let (mut server, mut client) = prepare_server_client(14);

    server.close();

    // Client will disconnect from server if connection lost.
    wait_update_message(&mut client, EthosClientUpdate::StatusChanged(EthosClientStatus::Disconnected));
}

#[test]
fn v17_client_message_too_large() {
    // V17 : ClientMessageTooLarge
    let (_server, mut client) = prepare_server_client(17);

    client.send(ClientMessage { size: (CLIENT_MSG_MAX_SIZE + 1) as u16 , payload: CONTROL_CLIENT_PAYLOAD }).unwrap();

    wait_update_message(&mut client, EthosClientUpdate::Error(ClientError::ClientMessageTooLarge));
}

#[test]
fn v18_update_vec() {
    //V18 : update_vec gives update in a convenient vector.
    let (_server, mut client) = prepare_server_client_no_wait(18);

    // Wait for connection messages
    thread::sleep(Duration::from_millis(100));

    let update = client.update_vec();
    assert!(update.len() > 0);

    // Wait for connection messages
    thread::sleep(Duration::from_millis(100));
    assert!(client.update_vec().len() == 0);    // Vec should be empty now.
}


#[test]
fn v19_message_vec() {
    // V19 : message_vec gives server message in a convenient vector.

    let (mut server, mut client) = prepare_server_client(19);

    let total_message : usize = u8::MAX as usize;

    // Send 256 messages
    for _ in 0..total_message {
        server.send_message(ServerMessage::new(CONTROL_SERVER_TIMESTAMP, CONTROL_SERVER_PAYLOAD));
    }

    // Wait for all message to reach client
    thread::sleep(Duration::from_millis(1000));

    match client.message_vec() {
        Ok(mvec) => {
            assert_eq!(mvec.len(), total_message);
            for msg in mvec {
                assert_eq!(*msg, ServerMessage::new(CONTROL_SERVER_TIMESTAMP, CONTROL_SERVER_PAYLOAD));
            }
        },
        Err(err) => panic!("v19 shouldn't err({:?})", err),
    }

    // Message vec should be empty now.
    assert_eq!(client.message_vec().unwrap().len(), 0);
    
}
