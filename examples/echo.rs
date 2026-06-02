use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Echo server listening on 127.0.0.1:8080");
    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("Accepted connection from: {}", addr);
        tokio::spawn(async move {
            let mut buf = [0;1024];
            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => break, 

                    Ok(n) => {
                        if let Err(e) = socket.write_all(&buf[..n]).await {
                            eprintln!("failed to write to the socket; err = {:?}", e);
                            break;
                        }
                    }

                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        break;
                    }
                }
            }
        });
    }
    Ok(())
}