//! Stores one or more elements.

use std::any::Any;
use std::cell::{OnceCell, Ref, RefCell, RefMut};
use std::collections::HashSet;
use std::path::Path;
use std::rc::{Rc, Weak};
use std::sync::Arc;

use craft_primitives::geometry::{Affine, Point, Rectangle};
use craft_renderer::renderer::Renderer;
use craft_resource_manager::{ResourceId, ResourceManager};

use maudio::engine::Engine;
use maudio::sound::Sound;
use maudio::sound::notifier::EndNotifier;

use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container, push_child_to_element};
use crate::elements::traits::DeepClone;
use crate::elements::{AsElement, Container, Element, ElementInternals, Slider, Text, TinyVg, resolve_clip_for_scrollable, scrollable};
use crate::events::{Event, EventKind};
use crate::layout::TaffyTree;
use crate::style::{AlignItems, Display, Overflow, Unit};
use crate::text::text_context::TextContext;
use crate::{Color, rgb};

#[derive(Clone)]
pub struct SoundData {
    sound: Rc<RefCell<Sound>>,
    end_notifier: EndNotifier,
}

pub struct AudioContext {
    pub engine: Engine,
    pub sounds: HashSet<u64>,
}

thread_local! {
    pub static AUDIO_CONTEXT: OnceCell<Rc<RefCell<AudioContext>>> = const { OnceCell::new() };
}

const PLAY: &[u8] = include_bytes!("../../../../assets/play.tvg");
const PAUSE: &[u8] = include_bytes!("../../../../assets/pause.tvg");
const VOLUME: &[u8] = include_bytes!("../../../../assets/volume.tvg");

#[derive(Clone)]
pub struct Audio {
    pub inner: Rc<RefCell<AudioInner>>,
}

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
#[derive(Clone)]
pub struct AudioInner {
    element_data: ElementData,
    play_button: TinyVg,
    track: Slider,
    controls: bool,
    play_icon: ResourceId,
    pause_icon: ResourceId,
    _volume_icon: ResourceId,
    volume_track: Slider,
    duration: Text,
    sound_data: Option<SoundData>,
}

impl Element for Audio {}

impl Drop for AudioInner {
    fn drop(&mut self) {
        ElementInternals::drop(self)
    }
}

impl AsElement for Audio {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.clone()
    }

    fn borrow(&self) -> Ref<'_, dyn ElementInternals> {
        self.inner.borrow()
    }

    fn borrow_mut(&self) -> RefMut<'_, dyn ElementInternals> {
        self.inner.borrow_mut()
    }
}

impl crate::elements::ElementData for AudioInner {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for AudioInner {
    fn deep_clone(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.deep_clone_internal()
    }

    fn apply_layout(
        &mut self,
        taffy_tree: &mut TaffyTree,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    ) {
        apply_generic_container_layout(
            self,
            taffy_tree,
            position,
            z_index,
            transform,
            text_context,
            clip_bounds,
            scale_factor,
        );
    }

    fn draw(
        &mut self,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        draw_generic_container(self, renderer, resource_manager, text_context, scale_factor);
    }

    fn on_event(
        &mut self,
        message: &EventKind,
        _text_context: &mut TextContext,
        event: &mut Event,
        _target: Option<Rc<RefCell<dyn ElementInternals>>>,
    ) {
        scrollable::handle_scroll_logic(self, message, event);
    }

    fn apply_clip(&mut self, clip_bounds: Option<Rectangle>) {
        let overflow = self.style().get_overflow();
        if overflow[0] == Overflow::Scroll || overflow[1] == Overflow::Scroll {
            resolve_clip_for_scrollable(self, clip_bounds);
        } else {
            self.element_data.layout.apply_clip(clip_bounds);
        }
    }

    fn push(&mut self, child: Rc<RefCell<dyn ElementInternals>>) {
        push_child_to_element(self, child);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Audio {
    pub fn new(path: &Path) -> Self {
        let play_icon = ResourceId::StaticBytes(PLAY);
        let pause_icon = ResourceId::StaticBytes(PAUSE);
        let volume_icon = ResourceId::StaticBytes(VOLUME);
        let play = TinyVg::new(play_icon.clone())
            .color(Color::WHITE)
            .width(Unit::Px(16.0))
            .height(Unit::Px(16.0));
        let track = Slider::new(16.0).width(Unit::Px(200.0)).thumb_color(Color::WHITE);
        let volume_track = Slider::new(16.0)
            .thumb_color(Color::WHITE)
            .min(0.0)
            .max(100.0)
            .value(100.0)
            .step(1.0);
        let duration = Text::new("").selectable(false).line_height(0.5).color(Color::WHITE);
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<AudioInner>>| {
            RefCell::new(AudioInner {
                element_data: ElementData::new(me.clone(), true),
                play_button: play.clone(),
                track: track.clone(),
                controls: true,
                play_icon,
                pause_icon,
                _volume_icon: volume_icon.clone(),
                duration: duration.clone(),
                volume_track: volume_track.clone(),
                sound_data: None,
            })
        });
        let inner2 = inner.clone();
        let inner3 = inner.clone();
        let inner4 = inner.clone();
        let mut inner_mut = inner.borrow_mut();
        inner_mut.set_height(Unit::Px(24.0));
        inner_mut.set_align_items(Some(AlignItems::Center));
        inner_mut.set_background_color(rgb(72, 72, 72));
        inner_mut.set_padding_all(Unit::Px(6.0));
        inner_mut.set_column_gap(Unit::Px(12.0));
        inner_mut.element_data.create_layout_node(None);
        inner_mut.push(
            Container::new()
                .push(play)
                .on_pointer_button_up(Rc::new(move |_event, _pb| {
                    inner2.borrow_mut().toggle();
                }))
                .inner,
        );
        inner_mut.push(
            track
                .on_slider_value_changed(Rc::new(move |_e, value| inner3.borrow_mut().set_cursor(value as f32)))
                .inner,
        );
        inner_mut.push(duration.inner);
        inner_mut.push(
            TinyVg::new(volume_icon)
                .color(Color::WHITE)
                .width(Unit::Px(16.0))
                .height(Unit::Px(16.0))
                .inner,
        );
        inner_mut.push(
            volume_track
                .on_slider_value_changed(Rc::new(move |_e, value| inner4.borrow_mut().set_volume(value as f32)))
                .inner,
        );
        inner_mut.set_sound(path);
        drop(inner_mut);
        Self { inner }
    }

    pub fn controls(self, controls: bool) -> Self {
        self.inner.borrow_mut().set_controls(controls);
        self
    }

    pub fn play(self) -> Self {
        self.inner.borrow_mut().play();
        self
    }

    pub fn pause(self) -> Self {
        self.inner.borrow_mut().pause();
        self
    }

    pub fn toggle(self) -> Self {
        self.inner.borrow_mut().toggle();
        self
    }

    pub fn is_playing(&self) -> bool {
        self.inner.borrow_mut().is_playing()
    }
}

/// Fetches or initializes the reference-counted AudioContext from thread-local storage.
fn get_context() -> Rc<RefCell<AudioContext>> {
    AUDIO_CONTEXT.with(|cell| {
        cell.get_or_init(|| {
            // Winit wants single threaded and miniaudio defaults to multithreaded.
            // If we use single threaded before miniaudio starts, it should safely fallback
            // to single threaded.
            use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
            unsafe {
                CoInitializeEx(None, COINIT_APARTMENTTHREADED).unwrap();
            }
            Rc::new(RefCell::new(AudioContext {
                engine: Engine::new().expect("Failed to create engine"),
                sounds: HashSet::new(),
            }))
        })
        .clone()
    })
}

impl AudioInner {
    fn toggle(&mut self) {
        let is_playing = self.is_playing();
        if is_playing {
            self.pause();
        } else {
            self.play();
        }
    }

    fn is_playing(&self) -> bool {
        if let Some(sound_data) = &self.sound_data {
            sound_data.sound.borrow().is_playing()
        } else {
            false
        }
    }

    fn set_sound(&mut self, path: &Path) {
        let ctx = get_context();
        let mut ctx = ctx.borrow_mut();
        let mut sound: Sound = ctx.engine.new_sound_from_file(path).unwrap();
        let end_notifier = sound.set_end_callback().unwrap();
        let duration = sound.length_seconds().unwrap_or_default() as f64;
        self.track.clone().max(duration);

        let current_time = sound.cursor_seconds().unwrap() as u32;
        let time = format_time(current_time, duration as u32);
        self.duration.clone().text(&time);
        ctx.sounds.insert(self.element_data.internal_id);
        // Todo: Clean up old sound data
        self.sound_data = Some(SoundData {
            sound: Rc::new(RefCell::new(sound)),
            end_notifier,
        });
        drop(ctx);
        self.set_volume(self.volume_track.get_value() as f32)
    }

    fn play(&self) {
        if let Some(sound_data) = &self.sound_data {
            self.play_button.clone().resource_id(self.pause_icon.clone());
            sound_data
                .sound
                .borrow_mut()
                .play_sound()
                .expect("Failed to play sound");
        }
    }

    fn pause(&self) {
        self.play_button.clone().resource_id(self.play_icon.clone());
        if let Some(sound_data) = &self.sound_data {
            sound_data
                .sound
                .borrow_mut()
                .stop_sound()
                .expect("Failed to pause sound");
        }
    }

    fn set_cursor(&self, value: f32) {
        if let Some(sound_data) = &self.sound_data {
            sound_data.sound.borrow_mut().seek_to_second(value).unwrap();
        }
    }

    fn set_volume(&self, value: f32) {
        let value = value / 100.0;
        if let Some(sound_data) = &self.sound_data {
            sound_data.sound.borrow_mut().set_volume(value);
        }
    }

    pub(crate) fn update(&self) {
        if let Some(sound_data) = &self.sound_data {
            let sound = sound_data.sound.borrow_mut();
            let current_time = sound.cursor_seconds().unwrap() as f64;
            let track = self.track.clone();

            let mut request_redraw = false;
            if track.get_value() != current_time {
                track.value(current_time);
                let total_time = sound.length_seconds().unwrap() as u32;
                let time = format_time(current_time as u32, total_time);
                self.duration.clone().text(&time);
                request_redraw = true;
            }
            sound_data.end_notifier.take_with(|| {
                self.play_button.clone().resource_id(self.play_icon.clone());
                request_redraw = true;
            });
            if request_redraw {
                self.request_window_redraw();
            }
        }
    }

    pub fn set_controls(&mut self, controls: bool) {
        self.controls = controls;
        if self.controls {
            self.set_display(Display::Flex);
        } else {
            self.set_display(Display::None);
        }
    }
}

fn format_time(current_time: u32, total_time: u32) -> String {
    let current_hours = current_time / 3600;
    let current_minutes = (current_time % 3600) / 60;
    let current_seconds = current_time % 60;

    let total_hours = total_time / 3600;
    let total_minutes = (total_time % 3600) / 60;
    let total_seconds = total_time % 60;

    if total_hours > 0 {
        format!(
            "{:02}:{:02}:{:02}/{:02}:{:02}:{:02}",
            current_hours, current_minutes, current_seconds, total_hours, total_minutes, total_seconds
        )
    } else {
        format!(
            "{:02}:{:02}/{:02}:{:02}",
            current_minutes, current_seconds, total_minutes, total_seconds
        )
    }
}
