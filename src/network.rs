#[path="network/http请求.rs"]
pub mod http请求;
#[path="network/http回应.rs"]
pub mod http回应;
pub mod router;
use std::{
    env, fs, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, path::PathBuf, sync::mpsc, thread, time::Duration
};
use project_root::get_project_root;
use http请求::{请求方法};
use http回应::{根据信息回复http报文, 根据文件路径回复http报文};
use crate::{login::{处理api_login请求, 处理api_register请求}, tool::log::{self, 日志信息, 日志生产者, 日志级别}, tool::thread_pool::线程池};

pub fn 绑定到端口(端口地址:&str){

    let listener = TcpListener::bind(端口地址).unwrap();
    log::初始化日志();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        线程池::get_instance().lock().unwrap().execute(||{
            
            处理http请求(stream);
        });
        //thread::spawn()
        
    }
}
pub fn 处理http请求(mut stream:TcpStream){
    let 日志="处理http请求".to_string();
    日志生产者::写入日志(日志, 日志级别::INFO);
    let mut read_buffer=[0;20000];
    stream.read(&mut read_buffer).unwrap();
    let mut http请求字符串=String::from_utf8(read_buffer.to_vec()).unwrap();
    
    if http请求字符串.is_empty(){
        return ;
    }
    let mut http请求:http请求::http请求=http请求字符串.into();

    router::router(stream,http请求);
    
}
