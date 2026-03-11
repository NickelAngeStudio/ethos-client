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

use crate::EthosNetClient;

impl EthosNetClient {

    /*
    fn handle_client_thread(connect_string : String, mut rcv_ctot : Receiver<CtoTMessage>, mut sdr_ttoc : Sender<TtoCMessage>) -> usize {

        // Put buffer on heap
        let mut buffer = Vec::<u8>::new();
        buffer.resize(SERVER_MSG_BUFFER_SIZE, 0);

        // Return value
        let mut return_value : usize = 0;

        match TcpStream::connect(connect_string){
            Ok(mut stream) => {
                stream.set_nonblocking(true).unwrap();
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

            },
            Err(_) => return_value = 2,
        }

        if return_value > 1 {
            debug_eprintln!("Thread error {}", return_value);
        }

        return_value

    }
    */
    
}