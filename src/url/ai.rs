use std::time::Duration;

use chrono::Local;
use mysql::{prelude::Queryable, Params};
use regex::Regex;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use crate::{deepseek_api, login::数据库登录查询信息, network::http请求::http请求, tool::{database::数据库连接池, log::{日志生产者, 日志级别}, tool::解析请求体json数据为结构体}};

use super::{diary_work::调整计划::{寻找所有计划, 更新完成任务情况}, long_work, 结果};
#[derive(Serialize,Deserialize,Debug)]
struct 请求体类{
    role:String,
    content:String,
}

#[derive(Serialize,Deserialize,Debug)]
struct deepseek请求json体类{
    model:String,
    messages:Vec<请求体类>,
    stream:bool,
}
#[derive(Serialize,Deserialize,Debug)]
struct deepseek回应json类{
    pub id: String,
    pub object: Option<String>,
    pub created: Option<i64>, 
    pub model: Option<String>,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
    pub system_fingerprint: Option<String>,
}
#[derive(Serialize,Deserialize, Debug)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub logprobs: Option<String>, // 使用 Option 表示可能为空的字段
    pub finish_reason: String,
}

#[derive(Serialize,Deserialize, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize,Deserialize, Debug)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub prompt_tokens_details: PromptTokensDetails,
    pub prompt_cache_hit_tokens: u32,
    pub prompt_cache_miss_tokens: u32,
}
#[derive(Serialize,Deserialize, Debug)]
pub struct PromptTokensDetails {
    pub cached_tokens: u32,
}
fn 截取sql命令(input: &str) -> (String, Vec<String>) {
    let re = Regex::new(r"##(.*?)##").unwrap();
    let mut extracted = Vec::new();
    let result = re.replace_all(input, |caps: &regex::Captures| {
        extracted.push(caps[1].to_string());
        "" // 替换为空字符串，即删除匹配到的内容
    }).to_string();
    (result, extracted)
}
pub fn 回应对话(信息:String,用户:数据库登录查询信息) ->结果<String>{
    let now = Local::now();
    // 提取日期部分（格式：YYYY-MM-DD）
    let current_date = now.format("%Y-%m-%d").to_string();
    
    let 回复体1=请求体类{role:"user".to_string(),content:信息};
    let 回复体2=请求体类{role:"system".to_string(),content:"当前系统有如下几张表
CREATE TABLE plan (
    username VARCHAR(255) NOT NULL,
    planname VARCHAR(255) NOT NULL
);这张是每日计划表 代表用户与每日要完成的计划
CREATE TABLE plan_work (
    username VARCHAR(255) NOT NULL,
    planname VARCHAR(255) NOT NULL,
    date DATE
);这张是每日完成计划日期
CREATE TABLE long_plan (
    username VARCHAR(255) NOT NULL,
    planname VARCHAR(255) NOT NULL,
    begin_date DATE,   // 开始日期
    end_date DATE,     // 结束日期
    progress INT,      // 完成进度 100代表完成
    is_solve BOOLEAN   // 是否完成
);这张是长期计划表,代表长期要完成的计划
当前用户为:".to_string()+&用户.username+"
他的每日计划为:"+&format!("{:?}",寻找所有计划(用户.username.clone())?)+
"他的长期计划为:"+&format!("{:?}",long_work::寻找所有计划(用户.username)?)+"\n"+"今天的日期为"+&current_date+r#"
如:增加计划 每天工作五小时 (当前用户为abc) 
则输出:
要求合法,正在执行命令 增加计划 每天工作五小时 ##insert into plan(username,planname) values ('abc','每天工作五小时')##
要求:命令以##号包裹 多条命令则由多个##号来包裹 并且中间不需要有任何转义符号 也不要有换行 不要用``` 输出不能只含有命令 要输出要求是否合法 并且对数据库干了什么
根据用户的说明生成sql语句 用##包裹 且生成的sql语句必须要仅仅操作有该用户名的数据 如果该用户想要操作别的用户的数据 拒绝该请求
如果用户没有明显操作数据库的意愿,则正常回答,不需要带##包裹的语句 如果用户有类似每天干什么什么目标多少天完成 则在每日计划和长期计划中都添加计划项 
如果用户说自己今天干了什么 要在每日计划和长期计划中寻找是否有该项并更改添加"#};
    let json请求=deepseek请求json体类{
        model:"deepseek-chat".to_string(),
        messages:vec![回复体1,回复体2],
        stream:false,
    };
    
    日志生产者::写入日志(format!("{:?}",json请求), 日志级别::DEBUG);
    let 回复=根据信息请求deepseek报文(json请求)?;
    
    Ok(回复.choices[0].message.content.clone())
    // Ok(回复.choices[0].message.content.clone())
}
pub fn 根据信息请求deepseek报文(deepseek请求json体:deepseek请求json体类)->结果<deepseek回应json类>{
    let client = Client::new();
    let post_url = "https://api.deepseek.com/chat/completions";
    let post_data = deepseek请求json体;
    let post_response = client.post(post_url)
                                    .header("Content-Type", "application/json")
                                    .header("Authorization", "Bearer ".to_owned()+deepseek_api)
                                    .timeout(Duration::from_secs(50))
                                    .json(&post_data) // 将数据序列化为 JSON
                                    .send()?;
    if post_response.status().is_success() {
        let body: deepseek回应json类 = post_response.json()?;
        println!("deepseek post成功:{:?}",body);
        日志生产者::写入日志(format!("deepseek post成功:{:?}",body), 日志级别::INFO);
        Ok(body)
    } else {
        日志生产者::写入日志(format!("deepseek post失败"), 日志级别::ERROR);
        Err("回复错误".into())
    }
}
#[derive(Serialize,Deserialize, Debug)]
struct 解析类{
    pub question:String
}
pub fn question_api(http请求:http请求,用户:数据库登录查询信息)->结果<String>{
    let 解析:解析类=解析请求体json数据为结构体(&http请求)?;
    let 回复字符串=回应对话(解析.question,用户.clone())?;
    let (回复字符串,sql命令)=截取sql命令(&回复字符串);
    for sql in sql命令 {
        //let 执行命令=&sql[2..=sql.len()-3];
        let mut conn=数据库连接池::get_instance().lock()?.get_conn()?;
        conn.exec_drop(sql.clone(),Params::Empty)?;
        日志生产者::写入日志(sql, 日志级别::DEBUG);
    }
    let now = Local::now();
    // 提取日期部分（格式：YYYY-MM-DD）
    let current_date = now.format("%Y-%m-%d").to_string();
    更新完成任务情况(&用户.username,&current_date)?;
    Ok(回复字符串.replace("\n", "<br>"))
}
#[cfg(test)]
mod test{
    use super::*;
    #[test]
    pub fn 测试deepseek连接是否成功(){
        let 回复体1=请求体类{role:"user".to_string(),content:"wer".to_string()};
        let json请求=deepseek请求json体类{
            model:"deepseek-chat".to_string(),
            messages:vec![回复体1],
            stream:false,
        };
        println!("{:?}",json请求);
        assert!(根据信息请求deepseek报文(json请求).is_ok());

    }
}
//deepseek请求json体类 { model: "deepseek-chat", messages: [请求体类 { role: "user", content: "wer" }], stream: false }
//deepseek请求json体类 { model: "deepseek-chat", messages: [请求体类 { role: "user", content: "wer" }], stream: false }