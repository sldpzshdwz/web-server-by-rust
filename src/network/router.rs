use std::{io::Write, net::TcpStream, thread, time::Duration};
use crate::{tool::tool::解析请求体json数据为结构体, url::{ai::question_api, diary_work::{self, 调整计划::{删除计划api, 增加计划api, 寻找所有计划, 提交完成计划api, 撤销完成计划api, 查询完成计划api, 查询某日的完成任务情况api}}, memory::{删除记忆选项, 删除记忆选项api, 增加记忆选项, 增加记忆选项api, 查找记忆选项, 获取复习记忆选项}, 结果}};
use cookie::{ time, Cookie};
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::{tool::database::数据库连接池, tool::log::{日志生产者, 日志级别}, login::{处理api_login请求, 处理api_register请求, 数据库登录查询信息,解析cookie中的jwt令牌}};

use super::{http回应::{根据信息回复http报文, 根据信息回复http报文并写入stream, 根据文件路径回复http报文, 根据文件路径回复http报文并写入stream}, http请求::{http请求, 请求方法}};
use std::cell::RefCell;


thread_local! {
    static 登录用户:RefCell<Option<数据库登录查询信息>> = RefCell::new(None);
}

pub fn 处理请求(mut stream:TcpStream,http请求:http请求){
    日志生产者::写入日志("url:".to_string()+&http请求.请求行.url, 日志级别::DEBUG);
    let mut 用户:Option<数据库登录查询信息>=None;
    if http请求.请求行.url.as_str()!="/" && http请求.请求行.url.as_str()!="/register"&& http请求.请求行.url.as_str()!="/api/login"&& http请求.请求行.url.as_str()!="/api/register"{
        if let Err(_)=解析cookie中的jwt令牌(&http请求,&mut 用户){
            let 回复报文=根据文件路径回复http报文("HTTP/1.1 500 not login","html/401.html");
                stream.write_all(回复报文.as_bytes()).unwrap();
            return;
        }
    }//检验登录信息是否正确
    let binding = http请求.clone();
    let 切割结果=binding.请求行.url.as_str().split('/').collect::<Vec<_>>();
    router(切割结果,http请求,stream,用户);
}
//根据文件路径回复http报文并写入stream(,,stream);
//根据信息回复http报文并写入stream(,,stream);
//判断是否是一级url并处理(切割结果,stream,http请求,用户,状态行,正文路径,执行下级路由)
pub fn router(切割结果:Vec<&str>,http请求:http请求,mut stream:TcpStream,用户:Option<数据库登录查询信息>){
    match  http请求.请求行.请求方法{
        请求方法::GET=> match 切割结果.clone()[1]{
            ""=>{根据文件路径回复http报文并写入stream("HTTP/1.1 200 OK","html/login.html",stream);},
            "register"=>{根据文件路径回复http报文并写入stream("HTTP/1.1 200 OK","html/register.html",stream);},
            "dashboard"=>{根据文件路径回复http报文并写入stream("HTTP/1.1 200 OK","html/dashboard.html",stream);},
            "diary_work"=>{判断是否是一级url并处理(切割结果,stream,http请求,用户,"HTTP/1.1 200 OK","html/diary_work.html",router_get_diary_work);},
            "memory"=>{判断是否是一级url并处理(切割结果,stream,http请求,用户,"HTTP/1.1 200 OK","html/memory.html",router_get_memory);}
            "api"=>{router_get_api(用户,stream,http请求,切割结果);},
            "ai"=>{判断是否是一级url并处理(切割结果,stream,http请求,用户,"HTTP/1.1 200 OK","html/ai.html",router_get_ai);},
            _=>{根据文件路径回复http报文并写入stream("HTTP/1.1 404 NOT FOUND","html/404.html",stream);}
        }
        请求方法::POST=>  match 切割结果.clone()[1]  {
            "api"=>router_post_api(用户,stream,http请求,切割结果),
            "diary_work"=>router_post_diary_work(用户,stream,http请求,切割结果),
            "ai"=>router_post_ai(用户,stream,http请求,切割结果),
            "memory"=>router_post_memory(用户,stream, http请求,切割结果),
            _=>{}
        }
        _=>{
        }
    }
}


//判断api处理是否成功并处理(,"HTTP/1.1 200 OK",,stream);
pub fn router_post_ai(用户:Option<数据库登录查询信息>,mut stream:TcpStream,http请求:http请求,切割结果:Vec<&str>){
    match 切割结果[2]{
        "question"=>{
            match question_api(http请求,用户.unwrap()){
                Ok(回应报文)=>{
                    根据信息回复http报文并写入stream("HTTP/1.1 200 OK",serde_json::to_string(&回应报文).unwrap(),stream);
                },
                Err(error)=>{
                    日志生产者::写入日志(error.to_string(), 日志级别::ERROR);
                }
            }
        }
        _=>{}
    }
}
pub fn router_get_ai(用户:Option<数据库登录查询信息>,mut stream:TcpStream,http请求:http请求,切割结果:Vec<&str>){
    match 切割结果[2]{
        _=>{根据文件路径回复http报文并写入stream("HTTP/1.1 404 NOT FOUND","html/404.html",stream);}
    }
}
pub fn router_get_api(用户:Option<数据库登录查询信息>,mut stream:TcpStream,http请求:http请求,切割结果:Vec<&str>){
    match 切割结果[2]{
        "get_username"=>{
            根据信息回复http报文并写入stream("HTTP/1.1 200 OK",serde_json::to_string(&用户.unwrap()).unwrap(),stream);
        },
        _=>{根据文件路径回复http报文并写入stream("HTTP/1.1 404 NOT FOUND","html/404.html",stream);}
    }
}
//判断api处理是否成功并将结果转换成json格式返回(api结果:Result<T,E>,状态行:&str,mut stream:TcpStream)
pub fn router_post_diary_work(用户:Option<数据库登录查询信息>,mut stream:TcpStream,http请求:http请求,切割结果:Vec<&str>){
    match 切割结果[2]{
        "delete_plan"=>{
            日志生产者::写入日志("运行删除api".to_string(), 日志级别::ERROR);
            判断api处理是否成功并处理(删除计划api(http请求),"HTTP/1.1 200 OK","".to_string(),stream);
        },
        "add_plan"=>{判断api处理是否成功并处理(增加计划api(http请求),"HTTP/1.1 200 OK","".to_string(),stream);},
        "solve_plan"=>{判断api处理是否成功并处理(提交完成计划api(http请求),"HTTP/1.1 200 OK","".to_string(),stream);},
        "revoke_solve_plan"=>{判断api处理是否成功并处理(撤销完成计划api(http请求),"HTTP/1.1 200 OK","".to_string(),stream);}
        "select_solve_plan"=>{判断api处理是否成功并将结果转换成json格式返回(查询完成计划api(http请求),"HTTP/1.1 200 OK",stream);}
        "select_solve_plan_data"=>{判断api处理是否成功并将结果转换成json格式返回(查询某日的完成任务情况api(http请求),"HTTP/1.1 200 OK",stream);}
        _=>{}
    }
}
pub fn router_get_diary_work(用户:Option<数据库登录查询信息>,mut stream:TcpStream,http请求:http请求,切割结果:Vec<&str>){
    
    match 切割结果[2]{
        "select_plan"=>{判断api处理是否成功并将结果转换成json格式返回(寻找所有计划(用户.unwrap().username),"HTTP/1.1 200 OK",stream);},
        "diary_work1"=>{根据文件路径回复http报文并写入stream("HTTP/1.1 200 OK","html/diary_work1.html",stream);},
        "diary_work2"=>{根据文件路径回复http报文并写入stream("HTTP/1.1 200 OK","html/diary_work2.html",stream);},
        "diary_work3"=>{根据文件路径回复http报文并写入stream("HTTP/1.1 200 OK","html/diary_work3.html",stream);},
        _=>{根据文件路径回复http报文并写入stream("HTTP/1.1 404 NOT FOUND","html/404.html",stream);}
    }
}
pub fn router_post_memory(用户:Option<数据库登录查询信息>,mut stream:TcpStream,http请求:http请求,切割结果:Vec<&str>){
    match 切割结果[2]{
        "add_memory"=>{
            match 增加记忆选项api(用户.unwrap(),http请求){
                Ok(())=>{
                    let 回复报文=根据信息回复http报文("HTTP/1.1 200 OK","".to_string());
                    stream.write_all(回复报文.as_bytes()).unwrap();
                },
                Err(error)=>{
                    日志生产者::写入日志(error.to_string(), 日志级别::ERROR);
                }
            };
        }
        "delete_memory"=>{
            match 删除记忆选项api(用户.unwrap(),http请求){
                Ok(())=>{
                    let 回复报文=根据信息回复http报文("HTTP/1.1 200 OK","".to_string());
                    stream.write_all(回复报文.as_bytes()).unwrap();
                },
                Err(error)=>{
                    日志生产者::写入日志(error.to_string(), 日志级别::ERROR);
                }
            };
        }
        "select_memory"=>{
            //日志生产者::写入日志("查找记忆选项".to_string(), 日志级别::ERROR);
            match 查找记忆选项(用户.unwrap(),http请求) {
                Ok(记忆选项)=>{
                    let 回复报文=根据信息回复http报文("HTTP/1.1 200 OK",serde_json::to_string(&记忆选项).unwrap());
                    日志生产者::写入日志(回复报文.clone(), 日志级别::DEBUG);
                    stream.write_all(回复报文.as_bytes()).unwrap(); 
                }
                Err(error)=>{
                    日志生产者::写入日志(error.to_string(), 日志级别::ERROR); 
                }   
            }
        }
        "get_memory_review"=>{
            match 获取复习记忆选项(用户.unwrap(),http请求) {
                Ok(记忆选项)=>{
                    let 回复报文=根据信息回复http报文("HTTP/1.1 200 OK",serde_json::to_string(&记忆选项).unwrap());
                    日志生产者::写入日志(回复报文.clone(), 日志级别::DEBUG);
                    stream.write_all(回复报文.as_bytes()).unwrap(); 
                }
                Err(error)=>{
                    日志生产者::写入日志(error.to_string(), 日志级别::ERROR); 
                }   
            }
        }
        _=>{根据文件路径回复http报文并写入stream("HTTP/1.1 404 NOT FOUND","html/404.html",stream);}
    }
}
pub fn router_get_memory(用户:Option<数据库登录查询信息>,mut stream:TcpStream,http请求:http请求,切割结果:Vec<&str>){

    match 切割结果[2]{
        "memory1"=>{根据文件路径回复http报文并写入stream("HTTP/1.1 200 OK","html/memory1.html",stream);}
        "memory2"=>{根据文件路径回复http报文并写入stream("HTTP/1.1 200 OK","html/memory2.html",stream);}
        _=>{根据文件路径回复http报文并写入stream("HTTP/1.1 404 NOT FOUND","html/404.html",stream);}
    }
}
pub fn router_post_api(用户:Option<数据库登录查询信息>,mut stream:TcpStream,http请求:http请求,切割结果:Vec<&str>){
    match 切割结果[2]{
        "login"=>{
            match 处理api_login请求(http请求) {
                Ok((用户信息,jwt_token))=>{
                    let cookie = Cookie::build(("jwt",jwt_token))
                        .path("/")
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
pub fn 判断是否是一级url并处理(切割结果:Vec<&str>,mut stream:TcpStream,http请求:http请求,用户:Option<数据库登录查询信息>,状态行:&str,正文路径:&str,执行下级路由:fn(Option<数据库登录查询信息>,TcpStream,http请求,Vec<&str>)){
    if 切割结果.len()<=2{
        根据文件路径回复http报文并写入stream(状态行, 正文路径, stream);
    }else{
        执行下级路由(用户,stream, http请求,切割结果);
    }
}
pub fn 判断api处理是否成功并处理<T,E:ToString>(api结果:Result<T,E>,状态行:&str,信息:String,mut stream:TcpStream){
    match api结果{
        Ok(_) => {根据信息回复http报文并写入stream(状态行,信息,stream);}
        Err(error) => {日志生产者::写入日志(error.to_string(), 日志级别::ERROR);}
    }
}
pub fn 判断api处理是否成功并将结果转换成json格式返回<T: Serialize,E:ToString>(api结果:Result<T,E>,状态行:&str,stream:TcpStream){
    match api结果{
        Ok(结果) => {根据信息回复http报文并写入stream(状态行,serde_json::to_string(&结果).unwrap(),stream);}
        Err(error) => {日志生产者::写入日志(error.to_string(), 日志级别::ERROR);}
    }
}