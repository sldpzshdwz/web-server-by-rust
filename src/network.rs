#[path="network/http请求.rs"]
pub mod http请求;
#[path="network/http回应.rs"]
pub mod http回应;
use std::{
    fs, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, thread, time::Duration
};
use crate::thread_pool::线程池;
pub fn 绑定到端口(端口地址:&str){

    let listener = TcpListener::bind(端口地址).unwrap();
    let mut thread_pool=线程池::new(1);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread_pool.execute(||{
            处理http请求(stream);
        });
        //thread::spawn()
        
    }
}
pub fn 处理http请求(mut stream:TcpStream){
    let reader=BufReader::new(&stream);
    // let http_request:Vec<_>=reader
    //         .lines()
    //         .map(|result| {result.unwrap()})
    //         .take_while(|result| {!result.is_empty()})
    //         .collect();
    let mut http请求:http请求::http请求;
    reader.read_to_string(http请求).unwrap();
    
    //println!("请求报文:\n{http_request:#?}");
    if http_request.is_empty() {
        return ;
    }
    let 切割请求头:Vec<_>=http_request[0][..].split(' ').collect();
    match  切割请求头[0]{
        "GET"=> match 切割请求头[1] {
            "/"=>{
                let 回复报文=回复报文("HTTP/1.1 200 OK","html/login.html");
                stream.write_all(回复报文.as_bytes()).unwrap();
            }
            "/sleep"=>{
                let 回复报文=回复报文("HTTP/1.1 200 OK","html/login.html");
                thread::sleep(Duration::from_millis(5000));
                stream.write_all(回复报文.as_bytes()).unwrap();
            }
            _=>{
                let 回复报文=回复报文("HTTP/1.1 404 NOT FOUND","html/404.html");
                stream.write_all(回复报文.as_bytes()).unwrap();
            }
        }
        "POST"=> match 切割请求头[1] {
            "/api/login"=>{
                
            }
            _=>{

            }
        }
        _=>{

        }
    }
    
}
fn 回复报文(状态行:&str,正文路径:&str) ->String{
    let 正文=fs::read_to_string(正文路径).unwrap();
    let 正文长度=正文.len();
    let 回复报文=format!("{状态行}\r\nContent-Length: {正文长度}\r\n\r\n{正文}");
    回复报文
}