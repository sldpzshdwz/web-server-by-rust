use std::error::Error;

use serde::{de::DeserializeOwned, Deserialize};

use crate::network::http请求::http请求;

use super::log::{日志生产者, 日志级别};

type 结果<T>=std::result::Result<T, Box<dyn Error>>;
pub fn 解析请求体json数据为结构体<'a,T: DeserializeOwned>(http请求:&'a http请求) ->结果<T>{
    let http请求体合并=http请求.请求体.join("\r\n");
    日志生产者::写入日志(http请求体合并.clone(), 日志级别::DEBUG);
    let 解析结果:T=serde_json::from_str::<T>(&http请求体合并)?;
    Ok(解析结果)
}