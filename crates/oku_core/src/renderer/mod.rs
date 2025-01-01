pub mod color;
pub mod renderer;

// software feature and not android
#[cfg(all(not(target_os = "android"), feature = "tinyskia_renderer"))]
pub mod softbuffer;

#[cfg(feature = "wgpu_renderer")]
pub mod wgpu;

#[cfg(feature = "vello_renderer")]
pub mod vello;

pub mod blank_renderer;
