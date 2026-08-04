use std::cell::RefCell;
use std::rc::{Rc, Weak};
#[cfg(not(target_arch = "wasm32"))]
use std::time;
use time::{Duration, Instant};
#[cfg(target_arch = "wasm32")]
use web_time as time;

use peniko::Color;
use taffy::AvailableSpace;

use retgui_primitives::brush::Brush;
use retgui_primitives::geometry::{Rectangle, Size};
use retgui_renderer::renderer::Renderer;
use retgui_renderer::text_renderer_data::TextData;
use crate::elements::{ElementInternals, Text};
use crate::text::text_context::TextContext;

const FPS_SAMPLE_INTERVAL: Duration = Duration::from_millis(1250);
const FPS_OVERLAY_PADDING_X: f32 = 8.0;
const FPS_OVERLAY_PADDING_Y: f32 = 5.0;
const FPS_OVERLAY_MARGIN: f32 = 8.0;

#[derive(Clone, Copy)]
pub(crate) struct LayoutStats {
    total: Duration,
    compute: Duration,
    apply: Duration,
}

#[derive(Clone, Copy)]
pub(crate) struct RenderStats {
    total: Duration,
    build_list: Duration,
    debug_overlay: Duration,
    sort: Duration,
    prepare: Duration,
    submit: Duration,
}

pub(crate) struct PerfStats {
    enabled: bool,
    text: Text,
    text_data: Weak<RefCell<dyn TextData>>,
    frames_since_sample: u32,
    sample_start: Instant,
    frame_total: Duration,
    layout: LayoutStats,
    render: RenderStats,
    scale_factor: f64,
}

impl Default for LayoutStats {
    fn default() -> Self {
        Self {
            total: Duration::from_secs(0),
            compute: Duration::from_secs(0),
            apply: Duration::from_secs(0),
        }
    }
}

impl LayoutStats {
    pub(crate) fn new(total: Duration, compute: Duration, apply: Duration) -> Self {
        Self {
            total,
            compute,
            apply,
        }
    }
}

impl Default for RenderStats {
    fn default() -> Self {
        Self {
            total: Duration::from_secs(0),
            build_list: Duration::from_secs(0),
            debug_overlay: Duration::from_secs(0),
            sort: Duration::from_secs(0),
            prepare: Duration::from_secs(0),
            submit: Duration::from_secs(0),
        }
    }
}

impl RenderStats {
    pub(crate) fn new(
        total: Duration,
        build_list: Duration,
        debug_overlay: Duration,
        sort: Duration,
        prepare: Duration,
        submit: Duration,
    ) -> Self {
        Self {
            total,
            build_list,
            debug_overlay,
            sort,
            prepare,
            submit,
        }
    }
}

impl PerfStats {
    pub(crate) fn new() -> Self {
        let text = Text::new("").selectable(false);
        {
            let mut text_inner = text.inner.borrow_mut();
            text_inner.set_font_size(16.0);
            text_inner.set_text_brush(Brush::Color(Color::WHITE));
        }
        let text_data_rc: Rc<RefCell<dyn TextData>> = text.inner.clone();
        let text_data = Rc::downgrade(&text_data_rc);

        let mut stats = Self {
            enabled: false,
            text,
            text_data,
            frames_since_sample: 0,
            sample_start: Instant::now(),
            frame_total: Duration::from_secs(0),
            layout: LayoutStats::default(),
            render: RenderStats::default(),
            scale_factor: 1.0,
        };
        stats.reset();
        stats
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub(crate) fn toggle(&mut self, renderer: &mut dyn Renderer) {
        self.enabled = !self.enabled;
        self.reset();
        renderer.set_vsync(!self.enabled);
    }

    pub(crate) fn update_stats(&mut self, total: Duration, layout: LayoutStats, render: RenderStats) {
        self.frame_total = total;
        self.layout = layout;
        self.render = render;
    }

    pub(crate) fn draw(
        &mut self,
        renderer: &mut dyn Renderer,
        text_context: &mut TextContext,
        scale_factor: f64,
    ) {
        if !self.enabled {
            return;
        }

        self.update_debug_text();
        let text_size = self.layout_text(text_context, scale_factor);

        renderer.start_overlay();
        let panel = Rectangle::new(
            FPS_OVERLAY_MARGIN,
            FPS_OVERLAY_MARGIN,
            text_size.width + FPS_OVERLAY_PADDING_X * 2.0,
            text_size.height + FPS_OVERLAY_PADDING_Y * 2.0,
        );
        renderer.draw_rect(
            panel.scale(scale_factor),
            Brush::Color(Color::from_rgba8(16, 16, 18, 215)),
        );
        renderer.draw_rect_outline(
            panel.scale(scale_factor),
            Brush::Color(Color::from_rgba8(255, 255, 255, 70)),
            scale_factor,
        );

        let text_rect = Rectangle::new(
            FPS_OVERLAY_MARGIN + FPS_OVERLAY_PADDING_X,
            FPS_OVERLAY_MARGIN + FPS_OVERLAY_PADDING_Y,
            text_size.width,
            text_size.height,
        );
        renderer.draw_text(self.text_data.clone(), text_rect.scale(scale_factor), None, false);
        renderer.end_overlay();
    }

    fn reset(&mut self) {
        self.frames_since_sample = 0;
        self.sample_start = Instant::now();
        self.frame_total = Duration::from_secs(0);
        self.layout = LayoutStats::default();
        self.render = RenderStats::default();
        let debug_text = self.debug_text(0.0);
        self.text.inner.borrow_mut().set_text(&debug_text);
    }

    fn update_debug_text(&mut self) {
        self.frames_since_sample += 1;
        let elapsed = self.sample_start.elapsed();
        if elapsed < FPS_SAMPLE_INTERVAL {
            return;
        }

        let fps = self.frames_since_sample as f64 / elapsed.as_secs_f64();
        self.frames_since_sample = 0;
        self.sample_start = Instant::now();
        let debug_text = self.debug_text(fps);
        self.text.inner.borrow_mut().set_text(&debug_text);
    }

    fn debug_text(&self, fps: f64) -> String {
        format!(
            "FPS: {fps:.0}\nframe: {}\nlayout: {}\n  compute: {}\n  apply: {}\nrender: {}\n  build list: {}\n  debug overlay: {}\n  sort: {}\n  prepare: {}\n  submit: {}",
            Self::format_duration(self.frame_total),
            Self::format_duration(self.layout.total),
            Self::format_duration(self.layout.compute),
            Self::format_duration(self.layout.apply),
            Self::format_duration(self.render.total),
            Self::format_duration(self.render.build_list),
            Self::format_duration(self.render.debug_overlay),
            Self::format_duration(self.render.sort),
            Self::format_duration(self.render.prepare),
            Self::format_duration(self.render.submit),
        )
    }

    fn format_duration(duration: Duration) -> String {
        let ms = duration.as_secs_f64() * 1000.0;
        if ms < 1.0 {
            let microseconds = duration.as_secs_f64() * 1_000_000.0;
            return if microseconds >= 10.0 || duration.is_zero() {
                format!("{microseconds:.0} μs")
            } else {
                format!("{microseconds:.1} μs")
            };
        }

        if ms >= 10.0 {
            format!("{ms:.1} ms")
        } else {
            format!("{ms:.2} ms")
        }
    }

    fn layout_text(&mut self, text_context: &mut TextContext, scale_factor: f64) -> Size<f32> {
        let mut text_inner = self.text.inner.borrow_mut();
        if (self.scale_factor - scale_factor).abs() > f64::EPSILON {
            text_inner.set_scale_factor(scale_factor);
            self.scale_factor = scale_factor;
        }

        let size = text_inner.measure(
            taffy::Size {
                width: None,
                height: None,
            },
            taffy::Size {
                width: AvailableSpace::MaxContent,
                height: AvailableSpace::MaxContent,
            },
            text_context,
        );
        text_inner
            .state
            .try_update_text_render(text_context, Brush::Color(Color::TRANSPARENT));

        Size::new(size.width, size.height)
    }
}
