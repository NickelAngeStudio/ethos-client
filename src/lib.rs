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

#[cfg(test)]
mod tests {
    use std::io::prelude::*;
    use std::net::TcpStream;

    use ethos_core::net::{ClientMessage, TCP_PORT};

    #[test]
    fn connection() {
        match create_connection(){
            Ok(_) => {},
            Err(_) => panic!("Connection error"),
        }
    }

    fn create_connection()  -> std::io::Result<()> {
        let connect_string = format!("127.0.0.1:{}", TCP_PORT);
        let buf =  &mut [0 as u8; 128];
        let mut stream = TcpStream::connect(connect_string)?;
        let mut count : u32 = 0;

        loop {
            if count < 10 {
                let msg = ClientMessage::new(ethos_core::net::ClientPayload::Key { key: 123 + count as u128 });
                match msg.pack_bytes(buf){
                    Ok(size) => {
                        println!("SIZE={}, Buf={:?}", size, buf);
                        stream.write(&buf[0..size]).unwrap();
                    },
                    Err(_) => panic!("Pack err"),
                }
                count += 1;
            }
        }

        
        //stream.read(&mut [0; 128])?;

        Ok(())
    }
}
