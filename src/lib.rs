use std::sync::{LazyLock, Mutex};

use tool::thread_pool::线程池;
use tool::log::日志级别;
pub mod network;
pub mod login;
pub mod tool;
pub mod url;
static 线程池线程数:u32=5;
static 数据库连接url: &str = "mysql://root:aA147741@localhost:3306/rust91";
static 日志显示级别:日志级别=日志级别::INFO;