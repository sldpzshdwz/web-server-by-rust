use std::{io::Write, net::TcpStream, thread, time::Duration};

use cookie::{ time, Cookie};
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::{database::数据库连接池, log::{日志生产者, 日志级别}, login::{处理api_login请求, 处理api_register请求, 数据库登录查询信息,解析cookie中的jwt令牌}};

use super::{http回应::{根据信息回复http报文, 根据文件路径回复http报文}, http请求::{http请求, 请求方法}};




pub fn router(mut stream:TcpStream,http请求:http请求){
    let mut 用户:Option<数据库登录查询信息>=None;
    if http请求.请求行.url.as_str()!="/" && http请求.请求行.url.as_str()!="/register"&& http请求.请求行.url.as_str()!="/api/login"&& http请求.请求行.url.as_str()!="/api/register"{
        if let Err(_)=解析cookie中的jwt令牌(&http请求,&mut 用户){
            let 回复报文=根据文件路径回复http报文("HTTP/1.1 500 not login","html/401.html");
                stream.write_all(回复报文.as_bytes()).unwrap();
        }
    }

    match  http请求.请求行.请求方法{
        请求方法::GET=> match http请求.请求行.url.as_str() {
            "/"=>{
                let 回复报文=根据文件路径回复http报文("HTTP/1.1 200 OK","html/login.html");
                stream.write_all(回复报文.as_bytes()).unwrap();
            },
            "/register"=>{
                let 回复报文=根据文件路径回复http报文("HTTP/1.1 200 OK","html/register.html");
                stream.write_all(回复报文.as_bytes()).unwrap();
            },
            "/dashboard"=>{
                let 回复报文=根据文件路径回复http报文("HTTP/1.1 200 OK","html/dashboard.html");
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
                    Ok((用户信息,jwt_token))=>{
                        let cookie = Cookie::build(("jwt",jwt_token))
                            .path("/")
                            .secure(true)
                            .max_age(cookie::time::Duration::minutes(5));
                        let 回复报文=根据信息回复http报文(&("HTTP/1.1 200 OK\r\nSet-Cookie: ".to_string()+&cookie.to_string())[..], r#"{"message":"成功"}"#.to_string());
                        
                        日志生产者::写入日志("登录成功,请求报文:\r\n".to_string()+"HTTP/1.1 200 OK\r\nSet-Cookie: "+&cookie.to_string(),日志级别::INFO);
                        stream.write_all(回复报文.as_bytes()).unwrap();
                    }
                    Err(信息)=>{
                        let 错误信息=format!("{:?}",信息);
                        let 回复报文=根据信息回复http报文("HTTP/1.1 500 FAIL", r#"{"message":"#.to_owned()+&错误信息+r#"}"#);
                        日志生产者::写入日志(r#"{"message":"#.to_owned()+&错误信息+r#"}"#,日志级别::INFO);
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