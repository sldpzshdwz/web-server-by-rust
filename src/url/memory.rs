use crate::{login::数据库登录查询信息, network::http请求::http请求, tool::{database::数据库连接池, tool::解析请求体json数据为结构体}};

use super::结果;
use mysql::{params, prelude::Queryable};
use serde::{de::value, Deserialize, Serialize};
// 定义表结构


// 定义统一的结构体
#[derive(Serialize, Deserialize,Debug,Clone)]
pub struct Memory {
    pub username: String,
    pub key: String,
    pub value: String,
    pub option1: String,
    pub option2: String,
    pub option3: String,
}

#[derive(Serialize, Deserialize,Debug)]
pub struct Memory_key {
    pub username: String,
    pub key: String,
}

#[derive(Serialize, Deserialize,Debug)]
pub struct memory_review {
    pub username: String,
    pub key: String,
    pub date: String,
    pub epoch: i32,
}
pub fn  增加记忆选项(username:String,key:String,value:String,option1:String,option2:String,option3:String) ->结果<()>{
    let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
    conn.exec_drop(r"insert into memory (username,`key`,value,option1,option2,option3) values (:username,:key,:value,:option1,:option2,:option3)",params!{
        "username"=>username,
        "key"=>key,
        "value"=>value,
        "option1"=>option1,
        "option2"=>option2,
        "option3"=>option3, 
    })?;
    Ok(())
}
pub fn 增加当日记忆复习选项(username:String,key:String) ->结果<()>{
    let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
    conn.exec_drop(r"insert into memory_review (username,`key`,date,epoch) values (:username,:key,CURDATE(),1)",params!{
        "username"=>username,
        "key"=>key, 
    })?;
    Ok(())
}
pub fn 增加记忆选项api(用户:数据库登录查询信息,http请求:http请求)->结果<()>{
    let memory=解析请求体json数据为结构体::<Memory>(&http请求)?;
    增加记忆选项(用户.username.clone(), memory.key.clone(), memory.value, memory.option1, memory.option2, memory.option3);
    增加当日记忆复习选项(用户.username, memory.key);
    Ok(())
}
pub fn  删除记忆选项(username:String,key:String) ->结果<()>{
    let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
    conn.exec_drop(r"delete from memory where username=:username and `key`=:key",params!{
        "username"=>username,
        "key"=>key,
    })?; 
    Ok(())
}
pub fn 删除记忆选项api(用户:数据库登录查询信息,http请求:http请求)->结果<()>{
    let memory=解析请求体json数据为结构体::<Memory_key>(&http请求)?;
    删除记忆选项(用户.username, memory.key)?;
    
    Ok(())
}
pub fn 查找记忆选项(用户:数据库登录查询信息,http请求:http请求)->结果<Vec<Memory>>{
    let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
    let 查询结果=conn.exec_map(r"select username,`key`,value,option1,option2,option3 from memory where username=:username",params!{
        "username"=>用户.username}, 
        |(username,key,value,option1,option2,option3)| {
            Memory { username, key, value, option1, option2, option3 }
        }
    )?;
    Ok(查询结果)
}
pub fn 查找记忆选项api(用户:数据库登录查询信息,http请求:http请求)->结果<Vec<Memory>>{
    查找记忆选项(用户,http请求)
}
const 复习间隔:[i32;8]=[1,1,1,2,3,7,15,30];
pub fn 获取复习记忆选项(用户:数据库登录查询信息,http请求:http请求)->结果<Memory>{
    let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
    let 查询结果=conn.exec_map(r"select username,`key`,date,epoch from memory_review WHERE username=:username and date <= CURDATE() limit 1",params!{
        "username"=>&用户.username}, 
        |(username,key,date,epoch): (Vec<u8>, Vec<u8>, mysql::Value, i32)| {
            memory_review{ username:String::from_utf8(username).unwrap(), key:String::from_utf8(key).unwrap() ,date:"".to_string(),epoch }
        }
    )?;
    if 查询结果.len()==0{
        return Ok(Memory{username:用户.username,key:"今日已无更多记忆任务".to_string(),value:"".to_string(),option1:"".to_string(),option2:"".to_string(),option3:"".to_string()});
    }
    conn.exec_drop(r"delete from memory_review where username=:username and `key`=:key",params!{
        "username"=>&用户.username,
        "key"=>&查询结果[0].key,
    })?;
    let key=查询结果[0].key.clone();
    let 查询结果2=conn.exec_map(r"select username,`key`,value,option1,option2,option3 from memory where username=:username and `key`=:key",params!{
        "username"=>&用户.username,
        "key"=>&key},
        |(username,key,value,option1,option2,option3)| {
            Memory { username, key, value, option1, option2, option3 }  
        } 
    )?;
    let 复习间隔日期=if 查询结果[0].epoch.clone()<=7{
        复习间隔[查询结果[0].epoch.clone() as usize]
    }else {
        30
    };
    if 查询结果2.len()!=0{
        conn.exec_drop(r"insert into memory_review (username,`key`,date,epoch) values (:username,:key,DATE_ADD(CURDATE(), INTERVAL :day DAY),:epoch)",params!{
            "username"=>&用户.username,
            "key"=>&key,
            "day"=>复习间隔日期,
            "epoch"=>查询结果[0].epoch.clone()+1,
        })?; 
        return Ok(查询结果2[0].clone());
    }else{
        return Ok(Memory{username:用户.username,key:"该问题已被删除".to_string(),value:"".to_string(),option1:"".to_string(),option2:"".to_string(),option3:"".to_string()});
    }
    Err("未实现".into())
}
