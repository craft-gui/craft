use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use std::task::{Context, Poll, Wake, Waker};

use crate::App;

type GuiAction = Box<dyn FnOnce(&mut App) + 'static>;
type GuiFuture = Pin<Box<dyn Future<Output = GuiAction> + 'static>>;

struct GuiWaker {
    callback: OnceLock<Box<dyn Fn() + Send + Sync + 'static>>,
}

impl Wake for GuiWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        if let Some(callback) = self.callback.get() {
            callback();
        }
    }
}

pub(crate) struct GuiActionQueue {
    futures: Vec<GuiFuture>,
    waker: Arc<GuiWaker>,
}

impl GuiActionQueue {
    pub(crate) fn new() -> Self {
        Self {
            futures: Vec::new(),
            waker: Arc::new(GuiWaker {
                callback: OnceLock::new(),
            }),
        }
    }

    pub(crate) fn spawn_local<F, O, C>(&mut self, future: F, on_complete: C)
    where
        F: Future<Output = O> + 'static,
        O: 'static,
        C: FnOnce(O, &mut App) + 'static,
    {
        self.futures.push(Box::pin(async move {
            let output = future.await;
            Box::new(move |app: &mut App| on_complete(output, app)) as GuiAction
        }));
        self.waker.wake_by_ref();
    }

    pub(crate) fn drain(&mut self) -> Vec<GuiAction> {
        let mut actions = Vec::new();
        let waker = Waker::from(self.waker.clone());
        let mut context = Context::from_waker(&waker);
        self.futures.retain_mut(|future| {
            if let Poll::Ready(action) = future.as_mut().poll(&mut context) {
                actions.push(action);
                false
            } else {
                true
            }
        });
        actions
    }

    pub(crate) fn set_waker(&self, waker: impl Fn() + Send + Sync + 'static) {
        let _ = self.waker.callback.set(Box::new(waker));
    }
}
