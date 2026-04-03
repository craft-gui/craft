mod run_later;

use std::future::Future;

use cfg_if::cfg_if;
pub use run_later::{pop_gui_thread_work, run_later_on_gui_thread};
pub use tokio::sync::mpsc::{Receiver, Sender, channel};
pub use tokio::*;

thread_local! {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) static LOCAL_SET: task::LocalSet = task::LocalSet::new();
}

pub struct CraftRuntime {
    #[cfg(not(target_arch = "wasm32"))]
    tokio_runtime: runtime::LocalRuntime,
}

#[derive(Clone)]
pub struct CraftRuntimeHandle {
    #[cfg(not(target_arch = "wasm32"))]
    tokio_runtime: runtime::Handle,
}

#[allow(clippy::derivable_impls)]
impl Default for CraftRuntime {
    fn default() -> Self {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                Self { }
            } else {
                Self {
                    tokio_runtime: runtime::LocalRuntime::new().expect("Failed to create tokio runtime."),
                }
            }
        }
    }
}

/// A cross-platform runtime for executing asynchronous tasks.
///
/// On non-WASM targets, it uses a Tokio runtime.
/// On WASM targets, it uses `wasm-bindgen-futures` to spawn local tasks.
///
/// To create a GUI that works on all platforms only use the spawn function.
/// For more advanced cases get the underlying runtime and downcast.
impl CraftRuntime {
    pub fn new() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                Self { }
            } else {
                Self {
                    tokio_runtime: runtime::LocalRuntime::new().expect("Failed to create tokio runtime."),
                }
            }
        }
    }

    pub fn handle(&self) -> CraftRuntimeHandle {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                CraftRuntimeHandle { }
            } else {
                CraftRuntimeHandle {
                    tokio_runtime: self.tokio_runtime.handle().clone(),
                }
            }
        }
    }

    #[allow(dead_code)]
    #[cfg(target_arch = "wasm32")]
    pub fn spawn<F>(future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        wasm_bindgen_futures::spawn_local(future)
    }

    #[allow(dead_code)]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn spawn<F>(future: F)
    where
        F: Future<Output = ()> + 'static + Send,
    {
        spawn(future);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn runtime_spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        #[cfg(not(target_arch = "wasm32"))]
        self.tokio_runtime.spawn_local(future);
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(future)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn borrow_tokio_runtime(&mut self) -> &mut runtime::LocalRuntime {
        &mut self.tokio_runtime
    }

    /// Block on or spawn a future on if blocking is not supported by the runtime.
    #[allow(dead_code)]
    #[cfg(target_arch = "wasm32")]
    pub fn maybe_block_on<F>(&self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        wasm_bindgen_futures::spawn_local(future)
    }

    /// Block on or spawn a future on if blocking is not supported by the runtime.
    #[allow(dead_code)]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn maybe_block_on<F>(&self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.tokio_runtime.block_on(future)
    }
}

impl CraftRuntimeHandle {
    pub fn update_local_set(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        LOCAL_SET.with(|local_set| {
            self.tokio_runtime.block_on(async {
                local_set
                    .run_until(async {
                        tokio::task::yield_now().await;
                    })
                    .await;
            });
        });
    }

    #[allow(dead_code)]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn spawn_current_thread<F>(&self, future: F)
    where
        F: Future + 'static,
        F::Output: 'static,
    {
        LOCAL_SET.with(|ls| ls.spawn_local(future));
    }

    #[allow(dead_code)]
    #[cfg(target_arch = "wasm32")]
    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        wasm_bindgen_futures::spawn_local(future)
    }

    #[allow(dead_code)]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + 'static + Send,
    {
        self.tokio_runtime.spawn(future);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn borrow_tokio_runtime(&mut self) -> &mut runtime::Handle {
        &mut self.tokio_runtime
    }

    /// Block on or spawn a future on if blocking is not supported by the runtime.
    #[allow(dead_code)]
    #[cfg(target_arch = "wasm32")]
    pub fn maybe_block_on<F>(&self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        wasm_bindgen_futures::spawn_local(future)
    }

    /// Block on or spawn a future on if blocking is not supported by the runtime.
    #[allow(dead_code)]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn maybe_block_on<F>(&self, future: F)
    where
        F: Future<Output = ()>,
    {
        self.tokio_runtime.block_on(future)
    }
}
