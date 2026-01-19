

#[cfg(test)]
mod tests {
    use std::io::prelude::*;
    use std::net::TcpStream;

    #[test]
    fn connection() {
        match create_connection(){
            Ok(_) => {},
            Err(_) => panic!("Connection error"),
        }
    }

    fn create_connection()  -> std::io::Result<()> {
        let mut stream = TcpStream::connect("127.0.0.1:3847")?;

        //stream.write(&[1])?;
        //stream.read(&mut [0; 128])?;

        Ok(())
    }
}
