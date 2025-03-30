use std::{cell::RefCell, fmt::Display, fs::{self, File}, path::PathBuf, sync::mpsc::{self, Receiver, Sender}, time::{SystemTime, UNIX_EPOCH}};

use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use project_root::get_project_root;
use std::io::Write;

use crate::tool::thread_pool::线程池;

#[derive(PartialEq)]
pub enum 日志级别{
    TRACE,
    DEBUG,
    INFO,
    WARN,
    ERROR,
}
impl 日志级别{
    pub fn 日志级别转化为数字(&self)->u32{
        match self{
            日志级别::TRACE => 0,
            日志级别::DEBUG => 1,
            日志级别::INFO => 2,
            日志级别::WARN => 3,
            日志级别::ERROR => 4,
        }
    }
}
pub struct 日志信息{
    pub message:String,
    pub 级别:日志级别,
    pub time_stamp:std::time::SystemTime,
}
impl Display for 日志信息{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let 级别=match self.级别{
            日志级别::TRACE => "TRACE",
            日志级别::DEBUG => "DEBUG",
            日志级别::INFO => "INFO",
            日志级别::WARN => "WARN",
            日志级别::ERROR => "ERROR",
        };
        let system_time = self.time_stamp;

        // 将 SystemTime 转换为 Unix 时间戳（秒数）
        let timestamp = system_time
            .duration_since(UNIX_EPOCH)
            .expect("获取时间戳失败")
            .as_secs();
        let offset = FixedOffset::east_opt(8 * 3600).unwrap();
        // 使用 chrono 将时间戳转换为人类可读的格式
        let naive_datetime = NaiveDateTime::from_timestamp(timestamp as i64, 0);
        let datetime: DateTime<FixedOffset> = DateTime::from_utc(naive_datetime, offset);
        let formatted_time = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
        write!(f, "[{}] {:?}:{}", 级别, formatted_time, self.message)
    }
}

pub struct 日志生产者{
    pub 发送通道:Option<Sender<日志信息>>
}
thread_local! {
    static 日志写入:RefCell<日志生产者> = RefCell::new(日志生产者 { 发送通道: None });
}
impl 日志生产者{
    pub fn 初始化(发送通道:Sender<日志信息>){
        日志写入.with(|obj|{obj.borrow_mut().发送通道=Some(发送通道)});
    }
    fn 生成日志(message:String,日志级别:日志级别)->日志信息{
        
        日志信息 { message, 级别: 日志级别, time_stamp: SystemTime::now() }
    }
    pub fn 写入日志(message:String,日志级别:日志级别){
        let 要发送的日志=Self::生成日志(message,日志级别);
        日志写入.with(|obj|{
            match obj.borrow().发送通道.as_ref(){
                Some(通道) =>{通道.send(要发送的日志);},
                None => {}
            };
        });
    }
}
pub struct 日志消费者{
    pub 接受通道:Option<Receiver<日志信息>>,
    pub 输出文件:File
}
impl 日志消费者{
    pub fn 初始化日志消费者(日志通道消费者:Receiver<日志信息>)->日志消费者{
        let 项目根路径: PathBuf = get_project_root()
            .expect("项目根路径解析错误")
            .into();

        // 构建完整路径
        let 日志路径 = 项目根路径.join("log").join("output.txt");

        // 确保父目录存在
        if let Some(父目录) = 日志路径.parent() {
            fs::create_dir_all(父目录).expect("无法创建日志目录");
        }

        // 创建文件
        let 输出文件 = File::create(&日志路径).expect("无法创建日志文件");
        日志消费者{ 接受通道:Some(日志通道消费者) ,输出文件:输出文件}
    }
    pub fn 处理日志并写入文件(&self){
        if let Some(接受端)=self.接受通道.as_ref(){
            loop{
                let 信息=接受端.recv();
                match 信息{
                    Ok(日志信息)=>{
                        if 日志信息.级别==日志级别::ERROR{
                            println!("\x1b[31m{}\x1b[0m",日志信息);
                        }else{
                            println!("{}",日志信息);
                        }
                        writeln!(&self.输出文件, "{}", 日志信息).expect("日志输出到文件错误");
                    }
                    Err(错误)=>{
                        break;
                    }
                }
            }
        }
    }
}
pub fn 初始化日志(){
    let (日志通道生产者,日志通道消费者)=mpsc::channel::<日志信息>();
    线程池::get_instance().lock().unwrap().初始化日志(日志通道生产者);
    线程池::get_instance().lock().unwrap().execute(||{
        let 日志消费者=日志消费者::初始化日志消费者(日志通道消费者);
        日志消费者.处理日志并写入文件();
    });
}
