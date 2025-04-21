use std::{error::Error, result};

use chrono::NaiveDate;
use mysql::{params, prelude::{FromRow, Queryable}, Row};
use serde::{de, Deserialize, Serialize};

use crate::{network::http请求::{http请求}, tool::{database::{数据库连接池, 用户数据}, log::{日志生产者, 日志级别}, tool::解析请求体json数据为结构体}, 数据库连接url};
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct 计划名{
    pub username:String,
    pub planname:String,
}

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct 计划{
    pub username:String,
    pub planname:String,
    pub begin_date:NaiveDate,
    pub end_date:NaiveDate,
    pub progress:i32,
    pub is_solve:bool
}

type 结果<T>=std::result::Result<T, Box<dyn Error>>;
pub fn 增加计划(计划:计划)->结果<()>{
    let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
    日志生产者::写入日志("增加计划".to_string(), 日志级别::INFO);
    conn.exec_drop(r"insert into long_plan (username,planname,begin_date,end_date,progress,is_solve) values (:username,:planname,:begin_date,:end_date,:progress,:is_solve)",params!{
        "username"=>计划.username,
        "planname"=>计划.planname,
        "begin_date"=>计划.begin_date.format("%Y-%m-%d").to_string(),
        "end_date"=>计划.end_date.format("%Y-%m-%d").to_string(),
        "progress"=>计划.progress,
        "is_solve"=>计划.is_solve,
    })?;
    Ok(())
}
pub fn 增加计划api(http请求:http请求)->结果<()>{
    let 解析结果:计划=解析请求体json数据为结构体::<计划>(&http请求)?;
    增加计划(解析结果);
    Ok(())
}
pub fn 删除计划(用户名:String,计划名:String)->结果<()>{
    日志生产者::写入日志(format!("删除计划启动 用户名:[{用户名}] 计划名:[{计划名}]"),日志级别::TRACE);
    let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
    conn.exec_drop(r"delete from long_plan where username=:username and planname=:planname",params!{
        "username"=>用户名,
        "planname"=>计划名,
    })?;
    Ok(())
}
pub fn 删除计划api(http请求:http请求)->结果<()>{
    let 解析结果:计划名=解析请求体json数据为结构体::<计划名>(&http请求)?;
    删除计划(解析结果.username,解析结果.planname)?;
    Ok(())
}
pub fn 寻找所有计划(用户名:String)->结果<Vec<计划>>{
    let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
    let 查询结果 = conn.exec_map(
        r"SELECT username, planname, begin_date, end_date, progress, is_solve 
          FROM long_plan 
          WHERE username = :username",
        params! { "username" => 用户名 },
        |mut row: Row| {
            let username: String = row.take("username").expect("无法获取 username");
            let planname: String = row.take("planname").expect("无法获取 planname");
            let begin_date: NaiveDate = row.take("begin_date").expect("无法获取 begin_date");
            let end_date: NaiveDate = row.take("end_date").expect("无法获取 end_date");
            let progress: i32 = row.take("progress").expect("无法获取 progress");
            let is_solve: i32 = row.take("is_solve").expect("无法获取 is_solve");

            // 将日期字符串解析为 NaiveDate
            // let begin_date = NaiveDate::parse_from_str(&begin_date_str, "%Y-%m-%d")
            //     .expect("无法解析 begin_date");
            // let end_date = NaiveDate::parse_from_str(&end_date_str, "%Y-%m-%d")
            //     .expect("无法解析 end_date");

            // 将整数转换为布尔值
            let is_solve = is_solve != 0;

            计划 {
                username,
                planname,
                begin_date,
                end_date,
                progress,
                is_solve,
            }
        }
    )?;
    println!("{:?}",查询结果);
    Ok(查询结果)
}
/*
let 查询结果=conn.exec_map(r"select username,planname,begin_date,end_date,progress,is_solve from long_plan where username=:username",params!{
            "username"=>用户名
        },
        |(username, planname,begin_date,end_date,progress,is_solve): (String, String, String, String, i32, i32)| {
            let begin_date=NaiveDate::parse_from_str(&begin_date, "%Y-%m-%d").unwrap();
            let end_date=NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").unwrap();
            let is_solve = is_solve != 0;
            计划 { username, planname ,begin_date,end_date,progress,is_solve}
        }
    )?;
*/
// CREATE TABLE plan (
//     username VARCHAR(255) NOT NULL,
//     planname VARCHAR(255) NOT NULL
// );
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct 完成计划{
    pub username:String,
    pub planname:String,
    pub progress:i32,
}
pub fn 提交完成计划api(http请求:http请求)->结果<()>{
    let 完成的计划:完成计划=解析请求体json数据为结构体::<完成计划>(&http请求)?;
    let 完成的计划克隆=完成的计划.clone();
    let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
    conn.exec_drop(r"UPDATE long_plan
SET progress =:progress
WHERE username = :username
AND planname = :planname;",params!{
        "username"=>完成的计划.username,
        "planname"=>完成的计划.planname,
        "progress"=>完成的计划.progress,
    })?;
    
    Ok(())
}
// pub fn 撤销完成计划api(http请求:http请求)->结果<()>{
//     let 完成的计划:计划名=解析请求体json数据为结构体::<计划名>(&http请求)?;
//     let 完成的计划克隆=完成的计划.clone();
//     let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
//     conn.exec_drop(r"UPDATE long_plan
// SET is_solve =FALSE
// WHERE username = :username
// AND planname = :planname;",params!{
//         "username"=>完成的计划.username,
//         "planname"=>完成的计划.planname,
//     })?;
//     Ok(())
// }
pub fn 查询完成计划api(http请求:http请求)->结果<i32>{
    let 完成的计划:完成计划=解析请求体json数据为结构体::<完成计划>(&http请求)?;
    let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
    let 查询结果=conn.exec_map(r"select progress from long_plan where username=:username and planname=:planname",params!{
            "username"=>完成的计划.username,
            "planname"=>完成的计划.planname,
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
        return Ok((1,100));
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