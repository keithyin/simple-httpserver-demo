mod thread_pool;

use std::{io, thread};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::net::Shutdown::Both;
use std::thread::sleep;
use std::time::Duration;
use crossbeam::channel;
use crate::thread_pool::ThreadPool;

fn handle_stream(mut stream: TcpStream) {
    println!("handle_stream begin");
    let mut buffer = [0; 4096];

    /// https://www.reddit.com/r/rust/comments/qm3loo/unexpected_behavior_with_tcpstream_set_to/
    /// 如果在这里设置 nonblocking的话，
    /// Err(ref e) if e.kind() == io::ErrorKind::WouldBlock 表示两层含义
    /// 1. tcp连接建立了，但是还没有发数据
    /// 2. 数据发完了
    /// 3. 还有没有第三种可能？有没可能数据发了一半，另一半等了好久还没到来？
    /// stream.set_nonblocking(true).unwrap(); // 如果没有设置nonblocking. 那么数据读完.read会block住

    loop {
        match stream.read(&mut buffer) { //
            Ok(n) => {
                let s = String::from_utf8_lossy(&buffer[..n]);
                // blocking状态的话，如果 客户端不断开连接，一直会阻塞在read方法上，所以这里如果确定读取结束，break就好了
                if s.ends_with("\r\n\r\n") {
                    break;
                }
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                /// 如果 nonblocking=true.
                ///     1. 数据读完 会走到该分支！
                ///     2. tcp连接构建成功，但是还没有数据发送过来。也会走到该分支。
                /// nonblocking=false
                ///     1. 应该是不会走到该分支了。
                break;
            },
            _ => {
                break;
            }
        }

        // stream.set_nonblocking(true).unwrap(); // 这里设计 nonblocking的话，可以保证一定是接收玩请求的数据才开始读的？

    }

    let response_str = "HTTP/1.1 200 OK

<!DOCTYPE html>
<html lang=\"en\">
<head>
<meta charset=\"udf-8\">
<title>Hello!</title>
</head>
<body>
HELLO
</body>
</html>
        ";
    stream.write(response_str.as_bytes()).unwrap();
    stream.flush().unwrap();

}

fn server() {
    let pool = ThreadPool::new(4);
    let listener = TcpListener::bind("127.0.0.1:9998").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(move || {
            handle_stream(stream);
        });
    }

}

// 启动后，浏览器 127.0.0.1:9998 进行访问
fn main() {
    server();
}
