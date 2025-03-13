use std::error::Error;
use serde::{Deserialize, Serialize};
use crate::network::http请求::{self, 请求方法};
use mysql::prelude;
use mysql::{self,params, prelude::Queryable};
#[derive(Serialize, Deserialize, Debug)]
struct 登录信息{
    username:String,
    password:String,
}
struct 数据库查询信息{
    username:String,
    password:String,
    permissions:String,
}

pub fn 处理api_login请求(http请求:http请求::http请求)->Result<(),Box<dyn Error>>{
    let http请求体合并=&http请求.请求体.join("\r\n");
    let 登录信息:登录信息=serde_json::from_str(http请求体合并)?;
    //查询登录信息是否在数据库中存在(登录信息)
    let mut conn = crate::database::数据库连接池::get_instance().lock().unwrap().get_conn().unwrap();
    let 查询结果=conn.exec_map(r"select username,password,permissions from users where username=:username",params!{
            "username"=>登录信息.username
        },
        |(username, password, permissions)| {
            数据库查询信息 { username, password, permissions }
        }
    )?;
    if 查询结果.len()==0 {
        return Err("未找到该用户".into());
    }
    if 查询结果[0].password!=登录信息.password{
        return Err("密码错误".into());
    }

    Ok(())
}
pub fn 处理api_register请求(http请求:http请求::http请求)->Result<(),Box<dyn Error>>{
    let http请求体合并=&http请求.请求体.join("\r\n");
    let 注册信息:登录信息=serde_json::from_str(http请求体合并)?;
    //查询登录信息是否在数据库中存在(登录信息)
    let mut conn = crate::database::数据库连接池::get_instance().lock().unwrap().get_conn().unwrap();
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
        let mut conn = crate::database::数据库连接池::get_instance().lock().unwrap().get_conn().unwrap();
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
                数据库查询信息 { username, password, permissions }
            }
        ).unwrap();

        assert_eq!(测试登录信息.len(),1);
        assert_eq!(测试登录信息[0].username,"keqing".to_string());
        assert_eq!(测试登录信息[0].password,"ganyu".to_string());
        assert_eq!(测试登录信息[0].permissions,"管理员".to_string());
    }
}
/*
CREATE TABLE users (
    username VARCHAR(255) NOT NULL,
    password VARCHAR(255) NOT NULL, -- 实际应用中请确保对密码进行加密处理
    permissions VARCHAR(255) NOT NULL,
    PRIMARY KEY (username)
);
 */