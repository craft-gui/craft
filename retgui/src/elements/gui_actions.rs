use std::future::Future;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, OnceLock};

use retgui_runtime::RetGuiRuntime;

use crate::App;

type GuiAction = Box<dyn FnOnce(&mut App) + 'static>;
type GuiWaker = Arc<dyn Fn() + Send + Sync + 'static>;

pub(crate) struct GuiActionQueue {
    sender: Sender<GuiAction>,
    receiver: Receiver<GuiAction>,
    waker: Arc<OnceLock<GuiWaker>>,
}

impl GuiActionQueue {
    pub(crate) fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            sender,
            receiver,
            waker: Arc::new(OnceLock::new()),
        }
    }

    pub(crate) fn spawn_local<F, O, C>(&self, future: F, on_complete: C)
    where
        F: Future<Output = O> + 'static,
        O: 'static,
        C: FnOnce(O, &mut App) + 'static,
    {
        let sender = self.sender.clone();
        let waker = self.waker.clone();
        RetGuiRuntime::spawn_local(async move {
            let output = future.await;
            if sender.send(Box::new(move |app| on_complete(output, app))).is_ok()
                && let Some(waker) = waker.get()
            {
                waker();
            }
        });
    }

    pub(crate) fn drain(&self) -> Vec<GuiAction> {
        self.receiver.try_iter().collect()
    }

    pub(crate) fn set_waker(&self, waker: impl Fn() + Send + Sync + 'static) {
        let _ = self.waker.set(Arc::new(waker));
    }
}
