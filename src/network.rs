use std::{
    fs, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}
};
pub fn 绑定到端口(端口地址:&str){

    let listener = TcpListener::bind(端口地址).unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        处理tcp请求(stream);
    }
}
pub fn 处理tcp请求(mut stream:TcpStream){
    let reader=BufReader::new(&stream);
    let http_request:Vec<_>=reader
            .lines()
            .map(|result| {result.unwrap()})
            .take_while(|result| {!result.is_empty()})
            .collect();
    //println!("请求报文:\n{http_request:#?}");
    match (&http_request[0][0..6]){
        "GET / "=>{
            let 状态行="HTTP/1.1 200 OK";
            let 正文=fs::read_to_string("html/index.html").unwrap();
            let 正文长度=正文.len();
            let 回复报文=format!("{状态行}\r\nContent-Length: {正文长度}\r\n\r\n{正文}");
            stream.write_all(回复报文.as_bytes()).unwrap();
        }
        _=>{
            let 状态行="HTTP/1.1 404 NOT FOUND";
            let 正文=fs::read_to_string("html/404.html").unwrap();
            let 正文长度=正文.len();
            let 回复报文=format!("{状态行}\r\nContent-Length: {正文长度}\r\n\r\n{正文}");
            stream.write_all(回复报文.as_bytes()).unwrap();
        }
    }
    
}
// fn 回复报文(状态行:&str,正文路径:&str,) ->String{
    
// }