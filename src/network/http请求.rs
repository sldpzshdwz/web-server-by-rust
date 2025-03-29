use std::collections::HashMap;

use crate::tool::log::{日志生产者, 日志级别};

#[derive(PartialEq,Debug,Clone)]
pub enum 请求方法{
    GET,
    POST,
    PUT,
    DELETE,
    OTHER,
    NONE
}

#[derive(PartialEq,Debug,Clone)]
pub enum http协议版本{
    HTTP1_0,
    HTTP1_1,
    HTTP2,
    OTHER,
    NONE
}

#[derive(PartialEq,Debug,Clone)]
pub struct 请求行{
    pub 请求方法:请求方法,
    pub url:String,
    pub http协议版本:http协议版本,
}
impl 请求行{
    fn default_new()->Self{
        请求行{
            请求方法:请求方法::NONE,
            url:"".to_string(),
            http协议版本:http协议版本::NONE
        }
    }
    fn new(s:&str)->Self{
        if s.is_empty(){
            日志生产者::写入日志("http请求为空".to_string(), 日志级别::ERROR);
        }
        let 切割请求行:Vec<_>=s.split(' ').collect();
        if 切割请求行.len()<=1{
            日志生产者::写入日志("错误解析的http报文".to_string()+s, 日志级别::ERROR);
            return 请求行{
                请求方法:请求方法::NONE,
                url:"".to_string(),
                http协议版本:http协议版本::NONE
            };
        }
        let 请求方法=match 切割请求行[0] {
            "GET"=>请求方法::GET,
            "POST"=>请求方法::POST,
            "DELETE"=>请求方法::DELETE,
            "PUT"=>请求方法::PUT,
            _=>请求方法::OTHER
        };
        let url=切割请求行[1].to_string();
        let http协议版本=match 切割请求行[2]{
            "HTTP/1.0"=>http协议版本::HTTP1_0,
            "HTTP/1.1"=>http协议版本::HTTP1_1,
            "HTTP/2"  =>http协议版本::HTTP2,
            _         =>http协议版本::OTHER,
        };
        请求行{
            请求方法,
            url,
            http协议版本
        }
    }
}
#[derive(PartialEq,Debug,Clone)]
pub struct http请求{
    pub 请求行:请求行,
    pub 请求头部:HashMap<String,String>,
    pub 请求体:Vec<String>
}
impl http请求{
    fn 从str转换到http请求(s:&str)->Self{
        //使用状态机
        let (mut 请求行,mut 请求头部,mut 请求体)=(请求行::default_new(),HashMap::<String,String>::new(),Vec::new());
        let mut 状态="请求行";
        let mut 请求体剩余长度:usize=0;
        for line in s.lines(){
            match 状态{
                "请求行"=>{
                    请求行=请求行::new(line);
                    状态="请求头部";
                },
                "请求头部"=>{
                    if line.is_empty(){
                        状态="请求体";
                        if let Some(i)=请求头部.get("Content-Length"){
                            请求体剩余长度=i[..].parse::<usize>().unwrap();
                        }
                    }else{
                        let 切分请求头部kv行:Vec<_>=line.split(":").collect();
                        请求头部.insert(切分请求头部kv行[0].trim().to_string(),切分请求头部kv行[1].trim().to_string());
                    }
                }
                "请求体" =>{
                    if line.is_empty(){
                        break;
                    }
                    let 有效字符:&str;
                    if line.len()>请求体剩余长度{
                        有效字符=&line[0..请求体剩余长度];
                    }else {
                        有效字符=&line[..];
                    }
                    请求体剩余长度-=有效字符.len();
                    请求体.push(有效字符.to_string());
                }         
                _=>{
                    panic!("状态机出现错误");
                }       
            }
        }
        http请求{
            请求行,
            请求头部,
            请求体
        }
    }
}
impl From<&str> for http请求{
    fn from(s: &str) -> Self{
        Self::从str转换到http请求(s)
    }
}
impl From<String> for http请求{
    fn from(s: String) -> Self{
        Self::从str转换到http请求(&s)
    }
}


#[cfg(test)]
mod test{
    use crate::network::http请求::{http协议版本, 请求方法};

    use super::http请求;

    #[test]
    fn 测试http请求解析正常情况(){
        let http请求字符串=
"POST /v1/resource HTTP/1.1
Host: api.example.com
Content-Type: application/json
User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3
Content-Length: 65

Hello 91";
        let http请求:http请求=http请求字符串.into();
        assert_eq!(http请求.请求行.请求方法,请求方法::POST);
        assert_eq!(http请求.请求行.url,"/v1/resource");
        assert_eq!(http请求.请求行.http协议版本,http协议版本::HTTP1_1);
        assert_eq!(http请求.请求头部["Host"],"api.example.com");
        assert_eq!(http请求.请求头部["Content-Type"],"application/json");
        assert_eq!(http请求.请求头部["User-Agent"],"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3");
        assert_eq!(http请求.请求头部["Content-Length"],"65");
        assert_eq!(http请求.请求体.len(),1);
        assert_eq!(http请求.请求体[0],"Hello 91");
    }
}