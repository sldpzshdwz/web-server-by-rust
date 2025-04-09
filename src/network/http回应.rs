use std::{fs, io::Write, net::TcpStream};

use project_root::get_project_root;
pub fn 根据文件路径回复http报文并写入stream(状态行:&str,正文路径:&str,mut stream:TcpStream){
    let 回复http页面报文=根据文件路径回复http报文(状态行,正文路径);
    stream.write_all(回复http页面报文.as_bytes()).unwrap(); 
}
pub fn 根据信息回复http报文并写入stream(状态行:&str,信息:String,mut stream:TcpStream){
    let 回复报文=根据信息回复http报文(状态行,信息);
    stream.write_all(回复报文.as_bytes()).unwrap();
}
pub fn 根据文件路径回复http报文(状态行:&str,正文路径:&str) ->String{
    //let 项目根路径 = env::var("CARGO_MANIFEST_DIR").expect("项目根路径解析错误");
    let 项目根路径: String= get_project_root().expect("项目根路径解析错误").into_os_string().into_string().expect("解析错误");
    let 正文=fs::read_to_string(项目根路径+"/"+正文路径).unwrap();
    let 正文长度=正文.len();
    let 回复http页面报文=format!("{状态行}\r\nContent-Length: {正文长度}\r\n\r\n{正文}");
    回复http页面报文
}

pub fn 根据信息回复http报文(状态行:&str,信息:String) ->String{
    //let 项目根路径 = env::var("CARGO_MANIFEST_DIR").expect("项目根路径解析错误");
    let 正文=信息;
    let 正文长度=正文.len();
    let 回复http页面报文=format!("{状态行}\r\nContent-Length: {正文长度}\r\n\r\n{正文}");
    回复http页面报文
}