#[path="network/http请求.rs"]
pub mod http请求;
#[path="network/http回应.rs"]
pub mod http回应;
use std::{
    env, fs, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, path::PathBuf, thread, time::Duration
};
use project_root::get_project_root;
use http请求::请求方法;

use crate::thread_pool::线程池;
pub fn 绑定到端口(端口地址:&str){

    let listener = TcpListener::bind(端口地址).unwrap();
    let mut thread_pool=线程池::new(3);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread_pool.execute(||{
            处理http请求(stream);
        });
        //thread::spawn()
        
    }
}
pub fn 处理http请求(mut stream:TcpStream){
    let mut reader=BufReader::new(&stream);
    let http_request:Vec<_>=reader
            .lines()
            .map(|result| {result.unwrap()})
            .take_while(|result| {!result.is_empty()})
            .collect();
    if http_request.is_empty(){
        return ;
    }
    let mut HttpString:String=http_request.join("\r\n");
    // let mut HttpString:String=String::new();

    // let result=reader.read_to_string(&mut HttpString);
    // if result.is_err(){
    //     return ;
    // }
    let mut http请求:http请求::http请求=HttpString.as_str().into();
    match  http请求.请求行.请求方法{
        请求方法::GET=> match http请求.请求行.url.as_str() {
            "/"=>{
                let 回复报文=回复报文("HTTP/1.1 200 OK","html/login.html");
                stream.write_all(回复报文.as_bytes()).unwrap();
            },
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
        请求方法::POST=>  match http请求.请求行.url.as_str()  {
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
    //let 项目根路径 = env::var("CARGO_MANIFEST_DIR").expect("项目根路径解析错误");
    let 项目根路径: String= get_project_root().expect("项目根路径解析错误").into_os_string().into_string().expect("解析错误");
    let 正文=fs::read_to_string(项目根路径+"/"+正文路径).unwrap();
    let 正文长度=正文.len();
    let 回复报文=format!("{状态行}\r\nContent-Length: {正文长度}\r\n\r\n{正文}");
    回复报文
}