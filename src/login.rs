use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use cookie::Cookie;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use crate::tool::log::{日志生产者, 日志级别};
use crate::network::http请求::{self, 请求方法};
use crate::tool::tool::解析请求体json数据为结构体;
use mysql::prelude;
use mysql::{self,params, prelude::Queryable};
#[derive(Serialize, Deserialize, Debug)]
struct 登录信息{
    username:String,
    password:String,
}
#[derive(Clone,Serialize)]
pub struct 数据库登录查询信息{
    pub username:String,
    pub password:String,
    pub permissions:String,
}
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    username:String,
    password:String,
    permissions:String,
    exp: usize,
}
pub fn 解析cookie中的jwt令牌(http请求:&http请求::http请求,用户:&mut Option<数据库登录查询信息>) -> Result<(),Box<dyn Error> >{
    if let Some(cookie)=http请求.请求头部.get("Cookie"){
        let cookie = Cookie::parse_encoded(cookie)?;
        let token=cookie.value();
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let token = decode::<Claims>(&token, &DecodingKey::from_secret(secret.as_ref()), &Validation::default())?;
        let 当前时间戳 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
        *用户=Some(数据库登录查询信息{
            username:token.claims.username.clone(),
            password:token.claims.password.clone(),
            permissions:token.claims.permissions.clone()
        });
        
        //日志生产者::写入日志(format!("当前时间戳:{} {:?}",当前时间戳,token.claims), 日志级别::DEBUG);
    }else {
        return Err("错误,没有cookie信息".into());
    }
    Ok(())
}
pub fn 处理api_login请求(http请求:http请求::http请求)->Result<(数据库登录查询信息,String),Box<dyn Error>>{
    // let http请求体合并=&http请求.请求体.join("\r\n");
    // let 登录信息:登录信息=serde_json::from_str(http请求体合并)?;
    //查询登录信息是否在数据库中存在(登录信息)
    let 登录信息:登录信息=解析请求体json数据为结构体::<登录信息>(&http请求)?;

    let mut conn = crate::tool::database::数据库连接池::get_instance().lock().unwrap().get_conn().unwrap();
    let 查询结果=conn.exec_map(r"select username,password,permissions from users where username=:username",params!{
            "username"=>登录信息.username
        },
        |(username, password, permissions)| {
            数据库登录查询信息 { username, password, permissions }
        }
    )?;
    if 查询结果.len()==0 {
        return Err("没有找到用户".into());
    }
    if 查询结果.len()>1{
        return Err("错误,查询到两个同名用户,请联系管理员".into());
    }
    if 查询结果[0].password!=登录信息.password{
        return Err("密码错误".into());
    }
    let 当前时间戳 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let 过期时间戳=当前时间戳+20*60;
    let my_claims=Claims{
        username:查询结果[0].username.clone(),
        password:查询结果[0].password.clone(),
        permissions:查询结果[0].permissions.clone(),
        exp:过期时间戳 as usize
    };
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = encode(&Header::default(), &my_claims, &EncodingKey::from_secret(secret.as_ref()))?;
    日志生产者::写入日志("jwt token:".to_string()+&token, 日志级别::INFO);

    Ok((查询结果[0].clone(),token))
}
pub fn 处理api_register请求(http请求:http请求::http请求)->Result<(),Box<dyn Error>>{
    let http请求体合并=&http请求.请求体.join("\r\n");
    let 注册信息:登录信息=serde_json::from_str(http请求体合并)?;
    //查询登录信息是否在数据库中存在(登录信息)
    let mut conn = crate::tool::database::数据库连接池::get_instance().lock().unwrap().get_conn().unwrap();
    conn.exec_drop(r"insert into users (username,password,permissions) values (:username,:password,:permissions)", params!{
        "username"=>注册信息.username,
        "password"=>注册信息.password,
        "permissions"=>"游客",
    })?;

    Ok(())
}
#[cfg(test)]
mod test{
    use mysql::Params;

    use crate::数据库连接url;

    use super::*;

    #[test]
    fn 测试json序列化(){
        let 登录信息:登录信息=serde_json::from_str(r##"{"username":"234243241324","password":"2342141324321"} "##).unwrap();
        assert_eq!(登录信息.username,"234243241324");
        assert_eq!(登录信息.password,"2342141324321");
    }
    #[test]
    fn 测试连接mysql数据库(){
        let mut conn = crate::tool::database::数据库连接池::get_instance().lock().unwrap().get_conn().unwrap();
        conn.exec_drop(r#"delete from users where username='keqing'"#,Params::Empty).unwrap();
        conn.exec_drop(r"insert into users (username,password,permissions) values (:username,:password,:permissions)", params!{
            "username"=>"keqing",
            "password"=>"ganyu",
            "permissions"=>"管理员",
        }).unwrap();
        let 测试登录信息 = conn
        .query_map(
            r#"SELECT username, password, permissions from users where username='keqing'"#,
            |(username, password, permissions)| {
                数据库登录查询信息 { username, password, permissions }
            }
        ).unwrap();

        assert_eq!(测试登录信息.len(),1);
        assert_eq!(测试登录信息[0].username,"keqing".to_string());
        assert_eq!(测试登录信息[0].password,"ganyu".to_string());
        assert_eq!(测试登录信息[0].permissions,"管理员".to_string());
    }
}
