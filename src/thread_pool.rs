use std::{cell::RefCell, error::Error, sync::{mpsc::{self, Sender}, Arc, Mutex, OnceLock}, thread::{self, JoinHandle}};

use crate::log::{日志信息, 日志生产者, 日志级别};

struct Worker{
    pub id:u32,
    pub join_handle:JoinHandle<()>,
    pub 通道生产者:Sender<Box<dyn FnOnce()+Send+'static>>,
    pub 未完成任务数:Arc<Mutex<u32>>
}
impl Worker{
    fn new(id:u32)->Worker{
        let (通道生产者,消费者)=mpsc::channel::<Box<dyn FnOnce()+Send+'static>>();
        let 未完成任务数:Arc<Mutex<u32>>=Arc::new(Mutex::new(0));
        let 未完成任务数克隆=未完成任务数.clone();
        let join_handle=thread::spawn(move ||{
            loop{
                let 信息=消费者.recv();
                match 信息{
                    Ok(函数)=>{
                        函数();
                        *(未完成任务数克隆.lock().unwrap())-=1;
                    },
                    Err(_)=>{
                        break;
                    }
                }
            }
        });
        Worker{id,join_handle,通道生产者,未完成任务数}
    }
    fn 派发工作(&self,f:Box<dyn FnOnce()+Send+'static>)->Result<(),Box<dyn Error>>{
        *(self.未完成任务数.lock().unwrap())+=1;
        self.通道生产者.send(f)?;
        
        Ok(())
    }
}
pub struct 线程池{
    线程:Vec<Worker>,
    线程数:u32,
}
impl 线程池{
    /**
     * 输入线程池数量
     * 生成一个线程池
     */
    pub fn new(线程数:u32)->线程池{
        let mut 线程=Vec::new();
        for i in 0..线程数{
            线程.push(Worker::new(i));
        }
        线程池{线程,线程数}
    }
    pub fn execute<F>(&mut self,f:F)
        where 
            F:FnOnce()+Send+'static,
        {
        let mut 最少任务数=99999999;
        let mut 最少任务工作者=0;
        for i in 0..self.线程数 as u32{
            let ans=*(self.线程[i as usize].未完成任务数.lock().unwrap());
            if ans<最少任务数 {
                最少任务数=ans;
                最少任务工作者=i;
            }
        }
        self.线程[最少任务工作者 as usize].派发工作(Box::new(f));
        let 日志=format!("派发工作到线程{最少任务工作者},当前线程还有{最少任务数}个未完成任务");
        日志生产者::写入日志(日志, 日志级别::INFO);
    }
    pub fn get_instance()->&'static Mutex<线程池>{
        static INSTANCE:OnceLock<Mutex<线程池>>=OnceLock::new();
        INSTANCE.get_or_init(|| Mutex::new(线程池::new(crate::线程池线程数)))
    }
    pub fn 初始化日志(&self,日志通道生产者:Sender<日志信息>){
        let 日志通道克隆生产者=日志通道生产者.clone();
        日志生产者::初始化(日志通道克隆生产者);
        for Worker in &self.线程{
            let 日志通道克隆生产者=日志通道生产者.clone();
            (*Worker).派发工作(Box::new(move ||{
                    日志生产者::初始化(日志通道克隆生产者);
                }
            )).unwrap();
        }
    }
}

#[cfg(test)]
mod test{
    use std::{thread::Thread, time::Duration};

    use super::*;
    #[test]
    fn 测试线程池_正常情况(){
        let mut pool=线程池::new(3);
        static mut TEST: Mutex<i32>=Mutex::new(0);
        
        for i in 0..100{
            let test=unsafe { & mut TEST };
            pool.execute(move||{
                for i in 0..3{
                    *(test.lock().unwrap())+=1;
                }
            });
            
        }
        std::thread::sleep(Duration::from_millis(10));
        unsafe{assert_eq!(*(TEST.lock().unwrap()),300);}
    }
}