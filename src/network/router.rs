use std::{io::Write, net::TcpStream, thread, time::Duration};
use crate::{tool::tool::解析请求体json数据为结构体, url::diary_work::{self, 调整计划::{删除计划api, 增加计划api, 寻找所有计划, 提交完成计划api, 撤销完成计划api, 查询完成计划api, 查询某日的完成任务情况api}}};
use cookie::{ time, Cookie};
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::{tool::database::数据库连接池, tool::log::{日志生产者, 日志级别}, login::{处理api_login请求, 处理api_register请求, 数据库登录查询信息,解析cookie中的jwt令牌}};

use super::{http回应::{根据信息回复http报文, 根据文件路径回复http报文}, http请求::{http请求, 请求方法}};
use std::cell::RefCell;


thread_local! {
    static 登录用户:RefCell<Option<数据库登录查询信息>> = RefCell::new(None);
}
pub fn router(mut stream:TcpStream,http请求:http请求){
    日志生产者::写入日志("url:".to_string()+&http请求.请求行.url, 日志级别::DEBUG);
    let mut 用户:Option<数据库登录查询信息>=None;
    if http请求.请求行.url.as_str()!="/" && http请求.请求行.url.as_str()!="/register"&& http请求.请求行.url.as_str()!="/api/login"&& http请求.请求行.url.as_str()!="/api/register"{
        if let Err(_)=解析cookie中的jwt令牌(&http请求,&mut 用户){
            let 回复报文=根据文件路径回复http报文("HTTP/1.1 500 not login","html/401.html");
                stream.write_all(回复报文.as_bytes()).unwrap();
            return;
        }
    }

    let binding = http请求.clone();
    let 切割结果=binding.请求行.url.as_str().split('/').collect::<Vec<_>>();
    match  http请求.请求行.请求方法{
        
        请求方法::GET=> match 切割结果.clone()[1]{
            ""=>{
                let 回复报文=根据文件路径回复http报文("HTTP/1.1 200 OK","html/login.html");
                stream.write_all(回复报文.as_bytes()).unwrap();
            },
            "register"=>{
                let 回复报文=根据文件路径回复http报文("HTTP/1.1 200 OK","html/register.html");
                stream.write_all(回复报文.as_bytes()).unwrap();
            },
            "dashboard"=>{
                let 回复报文=根据文件路径回复http报文("HTTP/1.1 200 OK","html/dashboard.html");
                stream.write_all(回复报文.as_bytes()).unwrap();
            },
            "diary_work"=>{
                if 切割结果.len()<=2{
                    let 回复报文=根据文件路径回复http报文("HTTP/1.1 200 OK","html/diary_work.html");
                    stream.write_all(回复报文.as_bytes()).unwrap();
                }else{
                    router_get_diary_work(用户,stream, http请求,切割结果);
                }
            },
            "api"=>{
                router_get_api(用户,stream,http请求,切割结果);
            },
            _=>{
                let 回复报文=根据文件路径回复http报文("HTTP/1.1 404 NOT FOUND","html/404.html");
                stream.write_all(回复报文.as_bytes()).unwrap();
            }
        }
        请求方法::POST=>  match 切割结果.clone()[1]  {
            "api"=>{
                router_post_api(用户,stream,http请求,切割结果);
            }
            "diary_work"=>{
                router_post_diary_work(用户,stream,http请求,切割结果);
            }
            _=>{

            }
        }
        _=>{

        }
    }
}
pub fn router_post_api(用户:Option<数据库登录查询信息>,mut stream:TcpStream,http请求:http请求,切割结果:Vec<&str>){
    match 切割结果[2]{
        "login"=>{
            match 处理api_login请求(http请求) {
                Ok((用户信息,jwt_token))=>{
                    let cookie = Cookie::build(("jwt",jwt_token))
                        .path("/")
                        .secure(true)
                        .max_age(cookie::time::Duration::minutes(20));
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
        "register"=>{
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
}
pub fn router_get_api(用户:Option<数据库登录查询信息>,mut stream:TcpStream,http请求:http请求,切割结果:Vec<&str>){
    match 切割结果[2]{
        "get_username"=>{
            let 回复报文=根据信息回复http报文("HTTP/1.1 200 OK",serde_json::to_string(&用户.unwrap()).unwrap());
            stream.write_all(回复报文.as_bytes()).unwrap();
        },
        _=>{
            let 回复报文=根据文件路径回复http报文("HTTP/1.1 404 NOT FOUND","html/404.html");
            stream.write_all(回复报文.as_bytes()).unwrap();
        }
    }
}
pub fn router_post_diary_work(用户:Option<数据库登录查询信息>,mut stream:TcpStream,http请求:http请求,切割结果:Vec<&str>){
    match 切割结果[2]{
        "delete_plan"=>{
            日志生产者::写入日志("运行删除api".to_string(), 日志级别::ERROR);
            match 删除计划api(http请求){
                Ok(())=>{
                    let 回复报文=根据信息回复http报文("HTTP/1.1 200 OK","".to_string());
                    stream.write_all(回复报文.as_bytes()).unwrap();
                },
                Err(error)=>{
                    日志生产者::写入日志(error.to_string(), 日志级别::ERROR);
                }
            }
        },
        "add_plan"=>{
            match 增加计划api(http请求){
                Ok(())=>{
                    let 回复报文=根据信息回复http报文("HTTP/1.1 200 OK","".to_string());
                    stream.write_all(回复报文.as_bytes()).unwrap();
                },
                Err(error)=>{
                    日志生产者::写入日志(error.to_string(), 日志级别::ERROR);
                }
            }
        },
        "solve_plan"=>{
            match 提交完成计划api(http请求){
                Ok(())=>{
                    let 回复报文=根据信息回复http报文("HTTP/1.1 200 OK","".to_string());
                    stream.write_all(回复报文.as_bytes()).unwrap();
                },
                Err(error)=>{
                    日志生产者::写入日志(error.to_string(), 日志级别::ERROR);
                }
            }
        },
        "revoke_solve_plan"=>{
            match 撤销完成计划api(http请求){
                Ok(())=>{
                    let 回复报文=根据信息回复http报文("HTTP/1.1 200 OK","".to_string());
                    stream.write_all(回复报文.as_bytes()).unwrap();
                },
                Err(error)=>{
                    日志生产者::写入日志(error.to_string(), 日志级别::ERROR);
                }
            }
        }
        "select_solve_plan"=>{
            match 查询完成计划api(http请求){
                Ok(查询数量)=>{
                    let 回复报文=根据信息回复http报文("HTTP/1.1 200 OK",serde_json::to_string(&查询数量).expect("解析查询数量失败"));
                    stream.write_all(回复报文.as_bytes()).unwrap();
                },
                Err(error)=>{
                    日志生产者::写入日志(error.to_string(), 日志级别::ERROR);
                }
            }
        }
        "select_solve_plan_data"=>{
            match 查询某日的完成任务情况api(http请求){
                Ok(任务完成情况)=>{
                    let 回复报文=根据信息回复http报文("HTTP/1.1 200 OK",serde_json::to_string(&任务完成情况).expect("解析查询数量失败"));
                    stream.write_all(回复报文.as_bytes()).unwrap();
                },
                Err(error)=>{
                    日志生产者::写入日志(error.to_string(), 日志级别::ERROR);
                }
            }
        }
        _=>{

        }
    }
}
pub fn router_get_diary_work(用户:Option<数据库登录查询信息>,mut stream:TcpStream,http请求:http请求,切割结果:Vec<&str>){
    
    match 切割结果[2]{
        "select_plan"=>{
            match 寻找所有计划(用户.unwrap().username){
                Ok(result)=>{
                    let 回复报文=根据信息回复http报文("HTTP/1.1 200 OK",serde_json::to_string(&result).expect("寻找所有计划转化为字符串失败"));
                    stream.write_all(回复报文.as_bytes()).unwrap();
                },
                Err(error)=>{
                    日志生产者::写入日志(error.to_string(), 日志级别::ERROR);
                }
            }
        },
        "diary_work1"=>{
            let 回复报文=根据文件路径回复http报文("HTTP/1.1 404 NOT FOUND","html/diary_work1.html");
            stream.write_all(回复报文.as_bytes()).unwrap();
        },
        "diary_work2"=>{
            let 回复报文=根据文件路径回复http报文("HTTP/1.1 404 NOT FOUND","html/diary_work2.html");
            stream.write_all(回复报文.as_bytes()).unwrap();
        },
        "diary_work3"=>{
            let 回复报文=根据文件路径回复http报文("HTTP/1.1 404 NOT FOUND","html/diary_work3.html");
            stream.write_all(回复报文.as_bytes()).unwrap();
        },
        _=>{
            let 回复报文=根据文件路径回复http报文("HTTP/1.1 404 NOT FOUND","html/404.html");
            stream.write_all(回复报文.as_bytes()).unwrap();
        }
    }
}

