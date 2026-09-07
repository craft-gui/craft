use std::any::Any;
use std::rc::Rc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

use gummy::{AvailableSpace, Size as GummySize};

use retgui_primitives::brush::Brush;
use retgui_primitives::geometry::{Rectangle, Size};

use retgui_renderer::renderer::Renderer;
use retgui_renderer::text_renderer_data::{TextData, TextSnapshot};

#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

use crate::Color;
use crate::accessibility::RetGuiAccessTree;
use crate::elements::{DynElement, ElementIds, ElementInternals, HasElementData as _, RetainedElements, TextElement};
use crate::layout::GummyTree;
use crate::text::text_context::TextContext;

const FPS_SAMPLE_INTERVAL: Duration = Duration::from_millis(250);
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
    text: DynElement,
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
    pub(crate) fn new(
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
    ) -> Self {
        let text = TextElement::insert(elements, gummy_tree, access_tree, by_internal_id, "");
        let text_element = elements.get_as_mut::<TextElement>(text);
        text_element.set_selectable(false);
        text_element.set_font_size(gummy_tree, 16.0);
        text_element.set_text_brush(gummy_tree, Brush::Color(Color::WHITE));
        let mut stats = Self {
            enabled: false,
            text,
            frames_since_sample: 0,
            sample_start: Instant::now(),
            frame_total: Duration::from_secs(0),
            layout: LayoutStats::default(),
            render: RenderStats::default(),
            scale_factor: 1.0,
        };
        stats.reset(elements, gummy_tree);
        stats
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub(crate) fn toggle(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        renderer: &mut dyn Renderer,
    ) {
        self.enabled = !self.enabled;
        self.reset(elements, gummy_tree);
        renderer.set_vsync(!self.enabled);
    }

    pub(crate) fn update_stats(&mut self, total: Duration, layout: LayoutStats, render: RenderStats) {
        self.frame_total = total;
        self.layout = layout;
        self.render = render;
    }

    pub(crate) fn draw(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        renderer: &mut dyn Renderer,
        text_context: &mut TextContext,
        scale_factor: f64,
    ) {
        if !self.enabled {
            return;
        }

        self.update_debug_text(elements, gummy_tree);
        let (text_size, snapshot) = self.layout_text(elements, gummy_tree, text_context, scale_factor);

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
        renderer.draw_text(snapshot, text_rect.scale(scale_factor), None, false);
        renderer.end_overlay();
    }

    fn reset(&mut self, elements: &mut RetainedElements, gummy_tree: &mut GummyTree) {
        self.frames_since_sample = 0;
        self.sample_start = Instant::now();
        self.frame_total = Duration::from_secs(0);
        self.layout = LayoutStats::default();
        self.render = RenderStats::default();
        let debug_text = self.debug_text(0.0);
        elements
            .get_as_mut::<TextElement>(self.text)
            .set_text(gummy_tree, &debug_text);
    }

    fn update_debug_text(&mut self, elements: &mut RetainedElements, gummy_tree: &mut GummyTree) {
        self.frames_since_sample += 1;
        let elapsed = self.sample_start.elapsed();
        if elapsed < FPS_SAMPLE_INTERVAL {
            return;
        }

        let fps = self.frames_since_sample as f64 / elapsed.as_secs_f64();
        self.frames_since_sample = 0;
        self.sample_start = Instant::now();
        let debug_text = self.debug_text(fps);
        elements
            .get_as_mut::<TextElement>(self.text)
            .set_text(gummy_tree, &debug_text);
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

    fn layout_text(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        text_context: &mut TextContext,
        scale_factor: f64,
    ) -> (Size<f32>, Rc<dyn TextData>) {
        let text = self.text;
        let scale_changed = (self.scale_factor - scale_factor).abs() > f64::EPSILON;
        self.scale_factor = scale_factor;
        elements.dispatch_mut(text, |element, elements| {
            let text_inner = (element as &mut dyn Any).downcast_mut::<TextElement>().unwrap();
            if scale_changed {
                text_inner.set_scale_factor(elements, gummy_tree, scale_factor);
            }
            let size = text_inner.measure(
                GummySize {
                    width: None,
                    height: None,
                },
                GummySize {
                    width: AvailableSpace::MaxContent,
                    height: AvailableSpace::MaxContent,
                },
                text_context,
            );
            let text_brush = text_inner.element_data().style.get_text_brush();
            text_inner
                .state
                .try_update_text_render(text_context, Brush::Color(Color::TRANSPARENT), text_brush, true);
            let snapshot: Rc<dyn TextData> = Rc::new(TextSnapshot::from_shared(
                text_inner
                    .state
                    .text_render
                    .clone()
                    .expect("performance text render not found"),
                text_inner.state.override_brush.clone(),
                true,
            ));
            (Size::new(size.width, size.height), snapshot)
        })
    }
}
