#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
use std::collections::VecDeque;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, Mutex, OnceLock};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct GuiThreadJobQueue {
    #[cfg(target_arch = "wasm32")]
    inner: VecDeque<Job>,
    #[cfg(not(target_arch = "wasm32"))]
    inner: Mutex<VecDeque<Job>>,
}

impl GuiThreadJobQueue {
    const fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        let inner = VecDeque::new();
        #[cfg(not(target_arch = "wasm32"))]
        let inner = Mutex::new(VecDeque::new());
        Self { inner }
    }
}

#[cfg(not(target_arch = "wasm32"))]
static GUI_THREAD_JOB_QUEUE: OnceLock<Arc<GuiThreadJobQueue>> = OnceLock::new();

#[cfg(target_arch = "wasm32")]
thread_local! {
    static GUI_THREAD_JOB_QUEUE: RefCell<GuiThreadJobQueue> = const { RefCell::new(GuiThreadJobQueue::new()) };
}

pub fn pop_gui_thread_work() -> Option<Job> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let queue = GUI_THREAD_JOB_QUEUE
            .get_or_init(|| Arc::new(GuiThreadJobQueue::new()))
            .clone();
        queue.pop()
    }

    #[cfg(target_arch = "wasm32")]
    {
        GUI_THREAD_JOB_QUEUE.with_borrow_mut(|queue| queue.pop())
    }
}

/// Runs a FnOnce at a later time on the GUI thread.
/// This is useful if you need to do work on another thread, but guarantee that GUI changes are done on the GUI thread.
/// This should only run for a very short time, because it will block the GUI from doing other work.
pub fn run_later_on_gui_thread(work: Job) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let queue = GUI_THREAD_JOB_QUEUE
            .get_or_init(|| Arc::new(GuiThreadJobQueue::new()))
            .clone();
        queue.push(work);
    }

    #[cfg(target_arch = "wasm32")]
    {
        GUI_THREAD_JOB_QUEUE.with_borrow_mut(|queue| {
            queue.push(work);
        });
    }
}

impl GuiThreadJobQueue {
    #[cfg(target_arch = "wasm32")]
    pub fn push(&mut self, item: Job) {
        self.inner.push_back(item);
    }

    #[cfg(target_arch = "wasm32")]
    pub fn pop(&mut self) -> Option<Job> {
        self.inner.pop_front()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn push(&self, item: Job) {
        let mut queue = self.inner.lock().unwrap();
        queue.push_back(item);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn pop(&self) -> Option<Job> {
        let mut queue = self.inner.lock().unwrap();
        queue.pop_front()
    }
}
