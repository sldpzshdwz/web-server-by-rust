use std::{error::Error, result};

use mysql::{params, prelude::Queryable};
use serde::{de, Deserialize, Serialize};

use crate::{network::http请求::{http请求}, tool::{database::{数据库连接池, 用户数据}, log::{日志生产者, 日志级别}, tool::解析请求体json数据为结构体}, 数据库连接url};

#[derive(Serialize, Deserialize, Debug)]
pub struct 计划{
    pub username:String,
    pub planname:String,
}
type 结果<T>=std::result::Result<T, Box<dyn Error>>;
pub fn 增加计划(用户名:String,计划名:String)->结果<()>{
    let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
    conn.exec_drop(r"insert into plan (username,planname) values (:username,:planname)",params!{
        "username"=>用户名,
        "planname"=>计划名,
    })?;
    Ok(())
}
pub fn 增加计划api(http请求:http请求)->结果<()>{
    let 解析结果:计划=解析请求体json数据为结构体::<计划>(&http请求)?;
    增加计划(解析结果.username,解析结果.planname);
    Ok(())
}
pub fn 删除计划(用户名:String,计划名:String)->结果<()>{
    日志生产者::写入日志(format!("删除计划启动 用户名:[{用户名}] 计划名:[{计划名}]"),日志级别::TRACE);
    let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
    conn.exec_drop(r"delete from plan where username=:username and planname=:planname",params!{
        "username"=>用户名,
        "planname"=>计划名,
    })?;
    Ok(())
}
pub fn 删除计划api(http请求:http请求)->结果<()>{
    let 解析结果:计划=解析请求体json数据为结构体::<计划>(&http请求)?;
    删除计划(解析结果.username,解析结果.planname)?;
    Ok(())
}
pub fn 寻找所有计划(用户名:String)->结果<Vec<计划>>{
    let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
    let 查询结果=conn.exec_map(r"select username,planname from plan where username=:username",params!{
            "username"=>用户名
        },
        |(username, planname)| {
            计划 { username, planname }
        }
    )?;
    println!("{:?}",查询结果);
    Ok(查询结果)
}
// CREATE TABLE plan (
//     username VARCHAR(255) NOT NULL,
//     planname VARCHAR(255) NOT NULL
// );
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct 完成计划{
    pub username:String,
    pub planname:String,
    pub date:String,
}
pub fn 更新完成任务情况(完成的计划:完成计划)->结果<()>{
    let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
    let 任务总数=conn.exec_map(r"select count(*) from plan where username=:username",params!{
        "username"=>完成的计划.username.clone()
    },
    |num:i32| {
        num
    })?;
    let 完成任务数=conn.exec_map(r"select count(*) from plan_work where username=:username and date=:date",params!{
        "username"=>完成的计划.username.clone(),
        "date"=>完成的计划.date.clone(),
    },
    |num:i32| {
        num
    })?;
    conn.exec_drop(r"delete from plan_work_data where username=:username and date=:date",params!{
        "username"=>完成的计划.username.clone(),
        "date"=>完成的计划.date.clone(),
    })?;
    conn.exec_drop(r"insert into plan_work_data (username,date,total_plan,complete_plan) values (:username,:date,:total_plan,:complete_plan)",params!{
        "username"=>完成的计划.username,
        "date"=>完成的计划.date,
        "total_plan"=>任务总数[0],
        "complete_plan"=>完成任务数[0],
    })?;
    Ok(())
}
pub fn 提交完成计划api(http请求:http请求)->结果<()>{
    let 完成的计划:完成计划=解析请求体json数据为结构体::<完成计划>(&http请求)?;
    let 完成的计划克隆=完成的计划.clone();
    let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
    conn.exec_drop(r"insert into plan_work (username,planname,date) values (:username,:planname,:date)",params!{
        "username"=>完成的计划.username,
        "planname"=>完成的计划.planname,
        "date"=>完成的计划.date,
    })?;
    更新完成任务情况(完成的计划克隆)?;
    Ok(())
}
pub fn 撤销完成计划api(http请求:http请求)->结果<()>{
    let 完成的计划:完成计划=解析请求体json数据为结构体::<完成计划>(&http请求)?;
    let 完成的计划克隆=完成的计划.clone();
    let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
    conn.exec_drop(r"delete from plan_work where username=:username and planname=:planname and date=:date",params!{
        "username"=>完成的计划.username,
        "planname"=>完成的计划.planname,
        "date"=>完成的计划.date,
    })?;
    更新完成任务情况(完成的计划克隆)?;
    Ok(())
}
pub fn 查询完成计划api(http请求:http请求)->结果<i32>{
    let 完成的计划:完成计划=解析请求体json数据为结构体::<完成计划>(&http请求)?;
    let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
    let 查询结果=conn.exec_map(r"select count(*) from plan_work where username=:username and planname=:planname and date=:date",params!{
            "username"=>完成的计划.username,
            "planname"=>完成的计划.planname,
            "date"=>完成的计划.date
        },
        |sl:i32| {
        sl
    })?;
    if (查询结果.len()==0){
        return Err("查询为空".into());
    }
    Ok(查询结果[0])
}
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct 查询完成任务情况类{
    pub username:String,
    pub date:String,
}
pub fn 查询某日的完成任务情况api(http请求:http请求)->结果<(i32,i32)>{
    let 完成任务情况:查询完成任务情况类=解析请求体json数据为结构体::<查询完成任务情况类>(&http请求)?;
    let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
    let 查询结果:Vec<(i32,i32)>=conn.exec_map(r"select complete_plan,total_plan from plan_work_data where username=:username and date=:date",params!{
        "username"=>完成任务情况.username,
        "date"=>完成任务情况.date
    },
    |num:(i32,i32)| {
        num
    })?;
    if (查询结果.len()==0){
        return Ok((0,1));
    }
    Ok(查询结果[0])
}
#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn 测试删除计划(){
    }
}