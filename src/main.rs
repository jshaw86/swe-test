extern crate tokio;

use tokio::{net::{TcpStream, TcpListener}, io::{AsyncWriteExt, AsyncReadExt}};

use std::result::Result;
use std::env;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    let upstream_server = env::var("UPSTREAM").unwrap(); 

    loop {
        println!("initialize stream");
        let (mut stream, _) = listener.accept().await.unwrap();
        
        println!("stream readable?");
        stream.readable().await.unwrap();

        println!("read all from stream");
        let buf = read_all(&stream).await.unwrap();
        // read request body from client
        // parse request body for later well 
        //println!("parse stream");
        //let _payload  = parse(&buf);
        // TODO: do stuff here

        // connect to upstream
        println!("connect to upstream");
        let mut upstream_stream = TcpStream::connect(&upstream_server).await.unwrap();

        println!("upstream writable?");
        upstream_stream.writable().await.unwrap();

        // write request body from client to upstream
        println!("write stream request body to upstream");
        upstream_stream.write_all(&buf).await.unwrap();

        println!("upstream readable?");
        upstream_stream.readable().await.unwrap();

        // read upstream response
        println!("read all from upstream");
        let upstream_buf = read_all(&upstream_stream).await.unwrap();
        let upstream_buf_str = std::str::from_utf8(&upstream_buf).unwrap().trim_matches(char::from(0));
        println!("upstream response {}", upstream_buf_str);

        println!("shutdown upstream connection");
        upstream_stream.shutdown().await.unwrap();

        println!("upstream writable?");
        stream.writable().await.unwrap();

        println!("write stream buf upstream response");
        stream.write_all(&upstream_buf_str.as_bytes()).await.unwrap();

        println!("shutdown stream");
        stream.shutdown().await.unwrap();
    }
}

/*
fn parse(buf: &Vec<u8>) -> Result<httparse::Status<usize>, httparse::Error> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);

    return req.parse(&buf)


}
*/


async fn read_all(stream: &TcpStream) -> Result<Vec<u8>, std::io::Error> {
    let mut buf: Vec<u8> = Vec::new(); 

    loop {
        // Creating the buffer **after** the `await` prevents it from
        // being stored in the async task.
        let mut tmp_buf = [0; 4096];

        // Try to read data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        match stream.try_read(&mut tmp_buf) {
            Ok(0) => break,
            Ok(_) => {
                buf.extend_from_slice(&tmp_buf)
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                break;
            }
            Err(e) => {
                return Err(e.into());
            }
        }

    }

    return Ok(std::str::from_utf8(&buf).unwrap().trim_matches(char::from(0)).as_bytes().to_vec())

}
