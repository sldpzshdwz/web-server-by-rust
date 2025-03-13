#[path="network/http请求.rs"]
pub mod http请求;
#[path="network/http回应.rs"]
pub mod http回应;
use std::{
    env, fs, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, path::PathBuf, sync::mpsc, thread, time::Duration
};
use project_root::get_project_root;
use http请求::{请求方法};
use http回应::{根据信息回复http报文, 根据文件路径回复http报文};
use crate::{api::{处理api_login请求, 处理api_register请求}, log::{self, 日志信息, 日志生产者, 日志级别}, thread_pool::线程池};

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

    match  http请求.请求行.请求方法{
        请求方法::GET=> match http请求.请求行.url.as_str() {
            "/"=>{
                let 回复报文=根据文件路径回复http报文("HTTP/1.1 200 OK","html/login.html");
                stream.write_all(回复报文.as_bytes()).unwrap();
            },
            "/sleep"=>{
                let 回复报文=根据文件路径回复http报文("HTTP/1.1 200 OK","html/login.html");
                thread::sleep(Duration::from_millis(5000));
                stream.write_all(回复报文.as_bytes()).unwrap();
            }
            "/dashboard"=>{
                let 回复报文=根据文件路径回复http报文("HTTP/1.1 200 OK","html/dashboard.html");
                stream.write_all(回复报文.as_bytes()).unwrap();
            }
            "/register"=>{
                let 回复报文=根据文件路径回复http报文("HTTP/1.1 200 OK","html/register.html");
                stream.write_all(回复报文.as_bytes()).unwrap();
            },
            _=>{
                let 回复报文=根据文件路径回复http报文("HTTP/1.1 404 NOT FOUND","html/404.html");
                stream.write_all(回复报文.as_bytes()).unwrap();
            }
        }
        请求方法::POST=>  match http请求.请求行.url.as_str()  {
            "/api/login"=>{
                match 处理api_login请求(http请求) {
                    Ok(())=>{
                        let 回复报文=根据信息回复http报文("HTTP/1.1 200 OK", r#"{"message":"成功"}"#.to_string());
                        stream.write_all(回复报文.as_bytes()).unwrap();
                    }
                    Err(信息)=>{
                        let 错误信息=format!("{:?}",信息);
                        let 回复报文=根据信息回复http报文("HTTP/1.1 500 FAIL", r#"{"message":"#.to_owned()+&错误信息+r#""}"#);
                        stream.write_all(回复报文.as_bytes()).unwrap();
                        println!("{信息}");
                    }
                }
            }
            "/api/register"=>{
                match 处理api_register请求(http请求) {
                    Ok(())=>{
                        let 回复报文=根据信息回复http报文("HTTP/1.1 200 OK", r#"{"message":"成功"}"#.to_string());
                        stream.write_all(回复报文.as_bytes()).unwrap();
                    }
                    Err(信息)=>{
                        let 错误信息=format!("{:?}",信息);
                        let 回复报文=根据信息回复http报文("HTTP/1.1 500 FAIL", r#"{"message":"#.to_owned()+&错误信息+r#""}"#);
                        stream.write_all(回复报文.as_bytes()).unwrap();
                        println!("{信息}");
                    }
                }
            }
            _=>{

            }
        }
        _=>{

        }
    }
    
}
