use cfg_if::cfg_if;
use std::future::Future;

pub struct CraftRuntime {
    #[cfg(not(target_arch = "wasm32"))]
    tokio_runtime: tokio::runtime::Runtime,
    #[cfg(target_arch = "wasm32")]
    #[allow(dead_code)]
    wasm_runtime: (),
}

#[allow(clippy::derivable_impls)]
impl Default for CraftRuntime {
    fn default() -> Self {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                Self { wasm_runtime: () }
            } else {
                Self { tokio_runtime: tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Failed to create tokio runtime.") }
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
                Self { wasm_runtime: () }
            } else if #[cfg(target_os = "macos")] {
                let tokio_runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime.");
            Self { tokio_runtime }
            } else {
                Self { tokio_runtime: tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Failed to create tokio runtime.") }
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
        tokio::spawn(future);
    }

    #[allow(dead_code)]
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn runtime_spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        wasm_bindgen_futures::spawn_local(future)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn runtime_spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + 'static + Send,
    {
        self.tokio_runtime.spawn(future);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn borrow_tokio_runtime(&mut self) -> &mut tokio::runtime::Runtime {
        &mut self.tokio_runtime
    }

    /// Match the underlying runtime's type.
    #[allow(dead_code)]
    #[cfg(target_arch = "wasm32")]
    pub fn native_spawn<F>(future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        wasm_bindgen_futures::spawn_local(future)
    }

    #[allow(dead_code)]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn native_spawn<F>(future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(future);
    }
}
