
use std::{thread, vec};
use std::thread::JoinHandle;
use crossbeam::channel;
use crossbeam::channel::Sender;

type Job = Box<dyn FnOnce() + Send + 'static>;
/// 维护一个线程池
/// 提交的任务放到一个队列里, 使用channel
/// 线程中的各个线程 从队列里面 取任务执行
/// 每个任务是一个 闭包！
pub struct ThreadPool {
    sender: Sender<Job>,
    threads: Vec<JoinHandle<()>>
}

impl ThreadPool {
    pub fn new(n: i32) -> Self{
        let (s, r) = channel::unbounded();
        let mut threads = vec![];
        for i in 0..n {
            let rc = r.clone();
            threads.push(thread::spawn(move || {
                loop {
                    let task: Job = rc.recv().unwrap();
                    println!("thread:{} received a job, processing...", i);
                    task();
                }

            }));
        }
        ThreadPool{sender: s, threads}
    }

    pub fn execute<F>(&self, f: F)
        where F: FnOnce() + Send + 'static {
        // 闭包是一个 ?Sized的，所以需要一个Box包装一下再塞到Channel里面去
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }



}