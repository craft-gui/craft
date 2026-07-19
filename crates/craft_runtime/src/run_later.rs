use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct Job {
    pub callback: JobCallback,
    pub interval: Option<Duration>,
    pub last_run: Instant
}

impl Job {
    pub fn new(callback: JobCallback, interval: Option<Duration>) -> Job {
        Job {
            callback,
            interval,
            last_run: Instant::now(),
        }
    }
}

type JobCallback = Box<dyn FnMut()>;

pub struct GuiThreadJobQueue {
    inner: VecDeque<Job>,
}

impl GuiThreadJobQueue {
    const fn new() -> Self {
        let inner = VecDeque::new();
        Self { inner }
    }
}
thread_local! {
  static GUI_THREAD_JOB_QUEUE: RefCell<GuiThreadJobQueue> = const { RefCell::new(GuiThreadJobQueue::new()) };
}

pub fn pop_gui_thread_work() -> Option<Job> {
    GUI_THREAD_JOB_QUEUE.with_borrow_mut(|queue| queue.pop())
}

pub fn push_gui_thread_work(work: Job) {
    GUI_THREAD_JOB_QUEUE.with_borrow_mut(|queue| {
        queue.push(work);
    });
}

/// Run the job at a later time. Can be on an interval or just once.
pub fn run_later(work: Job) {
    push_gui_thread_work(work);
}

impl GuiThreadJobQueue {

    pub fn push(&mut self, item: Job) {
        self.inner.push_back(item);
    }


    pub fn pop(&mut self) -> Option<Job> {
        self.inner.pop_front()
    }
}
