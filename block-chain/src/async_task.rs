use std::future::{Future};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::Duration;

trait WithArc<T> {
    fn new_arc(value: T) -> Arc<Mutex<Self>>;
    fn to_arc(&self) -> Arc<Mutex<&Self>> {
        Arc::new(Mutex::new(&self))
    }
}

impl WithArc<i8> for tokio::sync::Notify {
    fn new_arc(value: i8) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(tokio::sync::Notify::new()))
    }
}

impl WithArc<usize> for tokio::sync::Barrier {
    fn new_arc(n: usize) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(tokio::sync::Barrier::new(n)))
    }
}

pub async fn start_future_task<ResultType, F>(task: &mut F) -> ResultType
where
    ResultType: Send + Clone + 'static,
    F: TFutureTask<ResultType>,
{
    FutureTask::new(task, Some(Duration::from_millis(10))).await
}

pub fn get_future_result<F: Future>(future: F) -> F::Output {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(future)
}

#[derive(PartialEq)]
pub enum FutureTaskState {
    NoInit,
    Pending,
    Finish,
    Error,
}

pub struct FutureTaskContext<ResultType> {
    pub waker: Option<Waker>,
    pub state: FutureTaskState,
    pub result: Option<ResultType>,
}

pub trait TFutureTask<ResultType>: Send + Sync {
    fn start(&mut self) -> ResultType;
}

pub struct FutureTask<ResultType: Send> {
    pub duration: Option<Duration>,
    pub context: Arc<Mutex<FutureTaskContext<ResultType>>>,
}

impl<ResultType> FutureTask<ResultType>
where
    ResultType: Send + Clone + 'static,
{
    pub fn new<F: TFutureTask<ResultType>>(task: &mut F, duration: Option<Duration>) -> Self {
        println!("Task init");
        let context = Arc::new(Mutex::new(FutureTaskContext {
            state: FutureTaskState::NoInit,
            waker: None,
            result: None,
        }));
        let context_clone = context.clone();
        let result = task.start();
        let thread_handler = thread::spawn(move || {
            if let Some(d) = duration {
                thread::sleep(d);
            }
            let mut guard = context_clone.lock().unwrap();
            guard.state = FutureTaskState::Pending;
            guard.result = Some(result);
            guard.state = FutureTaskState::Finish;
            if let Some(waker) = guard.waker.take() {
                waker.wake();
            }
        });
        match thread_handler.join() {
            Ok(result) => {
                println!("Task result: {:?}", result);
            }
            Err(error) => {
                let mut guard = context.lock().unwrap();
                guard.state = FutureTaskState::Error;
                println!("Task error: {:?}", error);
            }
        }

        Self { duration, context }
    }
}

impl<ResultType> Future for FutureTask<ResultType>
where
    ResultType: Send + Clone + 'static,
{
    type Output = ResultType;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("Start poll");
        let mut guard = self.context.lock().unwrap();
        match guard.state {
            FutureTaskState::NoInit => {
                guard.waker = Some(cx.waker().clone());
            }
            FutureTaskState::Finish => {
                println!("Task finish");
                return Poll::Ready(guard.result.clone().unwrap());
            }
            _ => (),
        }
        if let Some(waker) = guard.waker.take() {
            if !waker.will_wake(&cx.waker()) {
                guard.waker = Some(cx.waker().clone());
            }
        }
        println!("End poll");
        Poll::Pending
    }
}

pub fn blocking_call() -> String {
    //  同步代码的睡眠
    std::thread::sleep(std::time::Duration::from_secs(3));
    "blocking call done".to_string()
}

pub async fn async_call(id: i32) -> i32 {
    //  异步睡眠任务
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    println!("async call done: ID: {}", id);
    id
}

pub async fn tokio_spawn() {
    let blocking_result = tokio::task::spawn_blocking(blocking_call).await.unwrap();
    print! {"blocking result: {}", blocking_result};
    let mut async_handlers = Vec::<tokio::task::JoinHandle<_>>::new();
    for i in 0..10i32 {
        async_handlers.push(tokio::task::spawn(async_call(i)));
    }
    for handler in async_handlers {
        let async_result = handler.await.unwrap();
        println! {"async result: {async_result}"};
    }
}

#[derive(Clone)]
pub struct BankWork<T> {
    semaphore_arc: Arc<tokio::sync::Mutex<tokio::sync::Semaphore>>,
    // handler: Box<dyn Fn(T) -> ()>,
    data: T,
}

impl<T> BankWork<T>
where
    T: Clone + Send + 'static,
{
    fn new(num: usize, data: T) -> Self {
        BankWork {
            semaphore_arc: Arc::new(tokio::sync::Mutex::new(tokio::sync::Semaphore::new(num))),
            data,
        }
    }

    pub async fn order<'a>(&mut self, name: &str) -> String {
        println!("{name} came to make order");
        self.teller(name).await;
        println!("{name} leave");
        String::from(name)
    }

    pub async fn order_in_spawn(&mut self, name: String) -> tokio::task::JoinHandle<String> {
        let _self = Arc::new(tokio::sync::Mutex::new(self.clone()));
        let _name = Arc::new(tokio::sync::Mutex::new(name));
        tokio::task::spawn(async move {
            // let mut that = that.lock().unwrap();
            let mut that = _self.lock().await;
            that.order(_name.lock().await.as_ref()).await
        })
    }

    pub async fn teller(&mut self, name: &str) {
        println!("order {name} waiting");
        let semaphore_arc = self
            .semaphore_arc
            .lock()
            .await;
        let permit = semaphore_arc.acquire().await.unwrap();
        println!("Teller work on order {name}");
        tokio::time::sleep(Duration::from_secs(5)).await;
        drop(permit);
        println!("Teller end work on order {name}");
    }
}



struct PackageWorker {
    notify_arc: Arc<Mutex<tokio::sync::Notify>>,
    barrier_arc: Arc<Mutex<tokio::sync::Barrier>>,
    handler: Box<dyn Fn(usize) -> ()>,
    is_start: bool,
}

impl PackageWorker {
    pub fn new(n: usize, handler: Box<dyn Fn(usize) -> ()>) -> Self {
        PackageWorker {
            notify_arc: tokio::sync::Notify::new_arc(0),
            barrier_arc: tokio::sync::Barrier::new_arc(n),
            handler,
            is_start: false,
        }
    }

    pub async fn push(&self) -> &Self {
        let wait_result = self.barrier_arc.lock().unwrap().wait().await;
        if wait_result.is_leader() {
            self.notify_arc.lock().unwrap().notify_one()
        }
        self
    }

    pub fn start(&mut self) -> Option<Box<dyn Future<Output = ()> + '_>> {
        if self.is_start {
            return Option::None;
        }
        self.is_start = true;
        Option::Some(Box::new(self.wait_for_notify()))
    }

    pub async fn wait_for_notify(&self) {
        while self.is_start {
            self.notify_arc.lock().unwrap().notified().await;
            self.handler()
        }
    }

    pub fn stop(&mut self) {
        self.is_start = false;
    }

    pub fn handler(&self) {}
}

struct RWDocument {
    rw_lock_arc: Arc<tokio::sync::RwLock<String>>,
}

impl RWDocument {
    pub fn new(value: String) -> Self {
        RWDocument {
            rw_lock_arc: Arc::new(tokio::sync::RwLock::new(value)),
        }
    }

    pub async fn read(&self) {
        let read_lock = self.rw_lock_arc.read().await;
    }

    pub async fn write(&self, value: String) {
        let mut write_lock = self.rw_lock_arc.write().await;
        write_lock.push_str(value.as_ref())
    }
}

#[cfg(test)]
mod async_task_tests {
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_tokio_spawn() {
        let fu = crate::async_task::tokio_spawn();
        sleep(Duration::from_secs(4));
    }

    #[test]
    fn test_bank_work() {
        let bank_work = crate::async_task::BankWork::new(4, Box::new(|_t: &str| {}));
        
        // Use tokio runtime to handle the async operations
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            for people in 0..10usize {
                let mut cloned_bank_work = bank_work.clone();
                let name = people.to_string();
                let handler = cloned_bank_work.order_in_spawn(name);
                let result = handler.await;
                println!("Handler completed");
            }
        });
    }

    #[test]
    fn test_batcher() {
        let limit = 12;
        let mut batcher = crate::async_task::PackageWorker::new(
            limit,
            Box::new(|index| println!("handler with index: {index}")),
        );
        for package in 0..60 {
            batcher.push();
        }
    }

    #[test]
    fn test_rw_document() {
        let rw_document = crate::async_task::RWDocument::new(String::from(""));
        tokio::task::spawn(async move {
            rw_document.read().await;
            rw_document.write(String::from("Start in here...")).await;
        });
    }
}
