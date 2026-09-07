//! An audio player element with optional controls.

use std::collections::VecDeque;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

use maudio::engine::Engine;
use maudio::sound::Sound;
use maudio::sound::notifier::EndNotifier;

use retgui_primitives::brush::Brush;

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::{ResourceId, ResourceManager};

#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container, push_child_to_element, queue_animation_update};
use crate::elements::traits::clone_element;
use crate::elements::{AnimationSchedule, Button, ButtonElement, DynElement, Element, ElementIds, ElementInternals, ElementStates, RetGuiAccessTree, RetainedElements, Slider, SliderElement, State, Text, TextElement, TinyVg, TinyVgElement, scrollable};
use crate::events::EventKind;
use crate::layout::GummyTree;
use crate::style::{AlignItems, Display, Unit};
use crate::text::text_context::TextContext;
use crate::{App, Color, ResourceType, rgb};

pub(crate) struct SoundData {
    sound: Sound,
    end_notifier: EndNotifier,
}

pub(crate) struct AudioContext {
    engine: Engine,
}

impl AudioContext {
    pub(crate) fn new() -> Self {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};

            unsafe {
                CoInitializeEx(None, COINIT_APARTMENTTHREADED).unwrap();
            }
        }

        Self {
            engine: Engine::new().expect("failed to create audio engine"),
        }
    }
}

/// Update interval for the audio controls while a sound is playing.
const AUDIO_UI_UPDATE_INTERVAL: Duration = Duration::from_millis(250);

const PLAY: &[u8] = include_bytes!("../../../assets/play.tvg");
const PAUSE: &[u8] = include_bytes!("../../../assets/pause.tvg");
const VOLUME: &[u8] = include_bytes!("../../../assets/volume.tvg");

#[derive(Clone, Copy)]
pub struct Audio {
    pub(crate) inner: DynElement,
}

#[derive(Clone)]
pub(crate) struct AudioElement {
    element_data: ElementData,
    play_button: Button,
    play_button_icon: TinyVg,
    track: Slider,
    controls: bool,
    play_icon: ResourceId,
    pause_icon: ResourceId,
    _volume_icon: ResourceId,
    volume_track: Slider,
    duration: Text,
    sound_data: Option<State<SoundData>>,
    next_ui_update: Option<Instant>,
}

impl Element for Audio {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl crate::elements::HasElementData for AudioElement {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for AudioElement {
    fn deep_clone(
        &self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
    ) -> DynElement {
        DynElement::new(clone_element::<Self, _>(
            self,
            elements,
            gummy_tree,
            access_tree,
            by_internal_id,
            |_, _| None,
        ))
    }

    fn apply_layout(
        &mut self,
        gummy_tree: &mut GummyTree,
        z_index: &mut u32,
        _text_context: &mut TextContext,
        scale_factor: f64,
    ) {
        apply_generic_container_layout(self, gummy_tree, z_index, scale_factor);
    }

    fn draw(
        &self,
        elements: &RetainedElements,
        states: &ElementStates,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        draw_generic_container(
            self,
            elements,
            states,
            renderer,
            resource_manager,
            text_context,
            scale_factor,
        );
    }

    fn on_event(
        &mut self,
        elements: &mut RetainedElements,
        _gummy_tree: &mut GummyTree,
        _access_tree: &RetGuiAccessTree,
        _by_internal_id: &mut ElementIds,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
        focus_outline_visible: bool,
        _pending_animation_updates: &mut Vec<(DynElement, bool)>,
        _states: &mut ElementStates,
        event: &mut EventKind,
        _text_context: &mut TextContext,
    ) {
        scrollable::handle_scroll_logic(elements, event_queue, focus, focus_outline_visible, self, event);
    }

    fn animation_tick(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        states: &mut ElementStates,
        pending_resources: &mut VecDeque<(ResourceId, ResourceType)>,
        delta: Duration,
    ) -> AnimationSchedule {
        let mut schedule = self.tick_style_animations(gummy_tree, delta);
        let Some(next_ui_update) = self.next_ui_update else {
            return schedule;
        };
        let now = Instant::now();
        if now >= next_ui_update {
            if self.update(elements, gummy_tree, states, pending_resources) {
                self.next_ui_update = None;
            } else {
                self.next_ui_update = Some(now + AUDIO_UI_UPDATE_INTERVAL);
            }
        }

        if let Some(next_ui_update) = self.next_ui_update {
            schedule = schedule.merge(AnimationSchedule::At(next_ui_update));
        }
        schedule
    }
}

impl Audio {
    pub fn new(app: &mut App, path: &Path) -> Self {
        let play_icon = ResourceId::StaticBytes(PLAY);
        let pause_icon = ResourceId::StaticBytes(PAUSE);
        let volume_icon = ResourceId::StaticBytes(VOLUME);
        let App {
            elements,
            gummy_tree,
            access_tree,
            by_internal_id,
            pending_resources,
            audio_context,
            states,
            ..
        } = app;

        let play_button = Button {
            inner: ButtonElement::insert(elements, gummy_tree, access_tree, by_internal_id),
        };
        elements
            .get_mut(play_button.inner)
            .element_data_mut()
            .set_accessibility_name("play");
        let play_button_icon = TinyVg {
            inner: TinyVgElement::insert(
                elements,
                gummy_tree,
                access_tree,
                by_internal_id,
                pending_resources,
                play_icon.clone(),
            ),
        };
        let icon = elements.get_mut(play_button_icon.inner);
        icon.set_text_brush(gummy_tree, Brush::Color(Color::WHITE));
        icon.set_width(gummy_tree, Unit::Px(16.0));
        icon.set_height(gummy_tree, Unit::Px(16.0));

        let track = Slider {
            inner: SliderElement::create(elements, gummy_tree, access_tree, by_internal_id, 16.0),
        };
        let track_element = elements.get_as_mut::<SliderElement>(track.inner);
        track_element.set_width(gummy_tree, Unit::Px(200.0));
        track_element.set_thumb_color(Brush::Color(Color::WHITE));
        let volume_track = Slider {
            inner: SliderElement::create(elements, gummy_tree, access_tree, by_internal_id, 16.0),
        };
        let volume = elements.get_as_mut::<SliderElement>(volume_track.inner);
        volume.set_thumb_color(Brush::Color(Color::WHITE));
        volume.set_min(0.0);
        volume.set_max(100.0);
        volume.set_value(100.0);
        volume.set_step(1.0);
        let duration = Text {
            inner: TextElement::insert(elements, gummy_tree, access_tree, by_internal_id, ""),
        };
        let duration_element = elements.get_as_mut::<TextElement>(duration.inner);
        duration_element.set_selectable(false);
        duration_element.set_line_height(gummy_tree, 0.5);
        duration_element.set_text_brush(gummy_tree, Brush::Color(Color::WHITE));

        let inner = elements.insert_with(access_tree, by_internal_id, |me, access_tree| {
            Box::new(AudioElement {
                element_data: ElementData::new(me, true, access_tree),
                play_button,
                play_button_icon,
                track,
                controls: true,
                play_icon,
                pause_icon,
                _volume_icon: volume_icon.clone(),
                volume_track,
                duration,
                sound_data: None,
                next_ui_update: None,
            })
        });
        let audio = elements.get_mut(inner);
        audio.set_height(gummy_tree, Unit::Px(24.0));
        audio.set_align_items(gummy_tree, AlignItems::Center);
        audio.set_background_brush(Brush::Color(rgb(72, 72, 72)));
        audio.set_padding_all(gummy_tree, Unit::Px(6.0));
        audio.set_column_gap(gummy_tree, Unit::Px(12.0));
        audio.element_data_mut().create_layout_node(gummy_tree, None);

        push_child_to_element(elements, gummy_tree, play_button.inner, play_button_icon.inner);
        elements
            .get_mut(play_button.inner)
            .on_click(Rc::new(move |_event, app| {
                app.elements.dispatch_mut(inner, |audio, arena| {
                    (audio as &mut dyn std::any::Any)
                        .downcast_mut::<AudioElement>()
                        .expect("audio handle changed type")
                        .toggle(
                            arena,
                            &mut app.gummy_tree,
                            &mut app.states,
                            &mut app.pending_resources,
                            &mut app.pending_animation_updates,
                        );
                });
            }));
        let play_control = play_button;
        elements
            .get_mut(track.inner)
            .on_slider_value_changed(Rc::new(move |event, app| {
                app.elements.get_as::<AudioElement>(inner).set_cursor(
                    &mut app.states,
                    app.elements.store_id(),
                    event.value as f32,
                );
            }));
        let track_control = track;
        elements
            .get_mut(volume_track.inner)
            .on_slider_value_changed(Rc::new(move |event, app| {
                app.elements.get_as::<AudioElement>(inner).set_volume(
                    &mut app.states,
                    app.elements.store_id(),
                    event.value as f32,
                );
            }));
        let volume_control = volume_track;
        let volume_icon_element = TinyVgElement::insert(
            elements,
            gummy_tree,
            access_tree,
            by_internal_id,
            pending_resources,
            volume_icon,
        );
        let icon = elements.get_mut(volume_icon_element);
        icon.set_text_brush(gummy_tree, Brush::Color(Color::WHITE));
        icon.set_width(gummy_tree, Unit::Px(16.0));
        icon.set_height(gummy_tree, Unit::Px(16.0));

        for child in [
            play_control.inner,
            track_control.inner,
            duration.inner,
            volume_icon_element,
            volume_control.inner,
        ] {
            push_child_to_element(elements, gummy_tree, inner, child);
        }
        elements.dispatch_mut(inner, |audio, elements| {
            (audio as &mut dyn std::any::Any)
                .downcast_mut::<AudioElement>()
                .expect("audio handle changed type")
                .set_sound(elements, gummy_tree, audio_context, states, path);
        });
        Self { inner }
    }

    pub fn set_controls(&self, app: &mut App, controls: bool) {
        let (gummy_tree, elements) = (&mut app.gummy_tree, &mut app.elements);
        if let Some(audio) = elements.try_get_as_mut::<AudioElement>(self.inner) {
            audio.set_controls(gummy_tree, controls);
        }
    }

    pub fn play(&self, app: &mut App) {
        app.elements.try_dispatch_mut(self.inner, |audio, arena| {
            (audio as &mut dyn std::any::Any)
                .downcast_mut::<AudioElement>()
                .expect("audio handle changed type")
                .play(
                    arena,
                    &mut app.gummy_tree,
                    &mut app.states,
                    &mut app.pending_resources,
                    &mut app.pending_animation_updates,
                );
        });
    }

    pub fn pause(&self, app: &mut App) {
        app.elements.try_dispatch_mut(self.inner, |audio, arena| {
            (audio as &mut dyn std::any::Any)
                .downcast_mut::<AudioElement>()
                .expect("audio handle changed type")
                .pause(
                    arena,
                    &mut app.gummy_tree,
                    &mut app.states,
                    &mut app.pending_resources,
                    &mut app.pending_animation_updates,
                );
        });
    }

    pub fn toggle(&self, app: &mut App) {
        app.elements.try_dispatch_mut(self.inner, |audio, arena| {
            (audio as &mut dyn std::any::Any)
                .downcast_mut::<AudioElement>()
                .expect("audio handle changed type")
                .toggle(
                    arena,
                    &mut app.gummy_tree,
                    &mut app.states,
                    &mut app.pending_resources,
                    &mut app.pending_animation_updates,
                );
        });
    }

    pub fn is_playing(&self, app: &App) -> bool {
        app.try_get_as::<AudioElement>(self.inner)
            .is_some_and(|audio| audio.is_playing(&app.states, app.elements.store_id()))
    }
}

impl AudioElement {
    fn toggle(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        states: &mut ElementStates,
        pending_resources: &mut VecDeque<(ResourceId, ResourceType)>,
        pending_animation_updates: &mut Vec<(DynElement, bool)>,
    ) {
        if self.is_playing(states, elements.store_id()) {
            self.pause(
                elements,
                gummy_tree,
                states,
                pending_resources,
                pending_animation_updates,
            );
        } else {
            self.play(
                elements,
                gummy_tree,
                states,
                pending_resources,
                pending_animation_updates,
            );
        }
    }

    fn is_playing(&self, states: &ElementStates, store_id: u64) -> bool {
        self.sound_data
            .map(|sound| sound.read_from(states, store_id).sound.is_playing())
            .unwrap_or(false)
    }

    fn set_sound(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        audio_context: &mut Option<AudioContext>,
        states: &mut ElementStates,
        path: &Path,
    ) {
        let (sound, end_notifier, duration, current_time) = {
            let context = audio_context.get_or_insert_with(AudioContext::new);
            let mut sound = context.engine.new_sound_from_file(path).unwrap();
            let end_notifier = sound.set_end_callback().unwrap();
            let duration = sound.length_seconds().unwrap_or_default() as f64;
            let current_time = sound.cursor_seconds().unwrap_or_default() as u32;
            (sound, end_notifier, duration, current_time)
        };

        elements.get_as_mut::<SliderElement>(self.track.inner).set_max(duration);
        elements
            .get_as_mut::<TextElement>(self.duration.inner)
            .set_text(gummy_tree, &format_time(current_time, duration as u32));
        self.sound_data = Some(State::insert(
            states,
            elements.store_id(),
            SoundData {
                sound,
                end_notifier,
            },
        ));
        let volume = elements.get_as::<SliderElement>(self.volume_track.inner).get_value() as f32;
        self.set_volume(states, elements.store_id(), volume);
    }

    fn play(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        states: &mut ElementStates,
        pending_resources: &mut VecDeque<(ResourceId, ResourceType)>,
        pending_animation_updates: &mut Vec<(DynElement, bool)>,
    ) {
        if let Some(button) = elements.try_get_mut(self.play_button.inner) {
            button.element_data_mut().set_accessibility_name("pause");
        }
        if let Some(icon) = elements.try_get_as_mut::<TinyVgElement>(self.play_button_icon.inner) {
            icon.set_resource_id(gummy_tree, pending_resources, self.pause_icon.clone());
        }
        if let Some(sound_data) = self.sound_data {
            sound_data
                .write_to(states, elements.store_id())
                .sound
                .play_sound()
                .expect("failed to play sound");
            self.start_progress_updates(pending_animation_updates);
        }
    }

    fn pause(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        states: &mut ElementStates,
        pending_resources: &mut VecDeque<(ResourceId, ResourceType)>,
        pending_animation_updates: &mut Vec<(DynElement, bool)>,
    ) {
        if let Some(button) = elements.try_get_mut(self.play_button.inner) {
            button.element_data_mut().set_accessibility_name("play");
        }
        if let Some(icon) = elements.try_get_as_mut::<TinyVgElement>(self.play_button_icon.inner) {
            icon.set_resource_id(gummy_tree, pending_resources, self.play_icon.clone());
        }
        if let Some(sound_data) = self.sound_data {
            sound_data
                .write_to(states, elements.store_id())
                .sound
                .stop_sound()
                .expect("failed to pause sound");
        }
        self.stop_progress_updates(pending_animation_updates);
    }

    fn set_cursor(&self, states: &mut ElementStates, store_id: u64, value: f32) {
        if let Some(sound_data) = self.sound_data {
            sound_data
                .write_to(states, store_id)
                .sound
                .seek_to_second(value)
                .unwrap();
        }
    }

    fn set_volume(&self, states: &mut ElementStates, store_id: u64, value: f32) {
        if let Some(sound_data) = self.sound_data {
            sound_data.write_to(states, store_id).sound.set_volume(value / 100.0);
        }
    }

    fn update(
        &self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        states: &mut ElementStates,
        pending_resources: &mut VecDeque<(ResourceId, ResourceType)>,
    ) -> bool {
        let Some(sound_data) = self.sound_data else {
            return false;
        };
        let mut ended = false;
        let (current_time, total_time) = {
            let sound_data = sound_data.write_to(states, elements.store_id());
            let current_time = sound_data.sound.cursor_seconds().unwrap_or_default() as f64;
            let total_time = sound_data.sound.length_seconds().unwrap_or_default() as u32;
            sound_data.end_notifier.take_with(|| ended = true);
            (current_time, total_time)
        };

        if elements
            .try_get_as::<SliderElement>(self.track.inner)
            .map_or(0.0, SliderElement::get_value)
            != current_time
        {
            if let Some(track) = elements.try_get_as_mut::<SliderElement>(self.track.inner) {
                track.set_value(current_time);
            }
            if let Some(duration) = elements.try_get_as_mut::<TextElement>(self.duration.inner) {
                duration.set_text(gummy_tree, &format_time(current_time as u32, total_time));
            }
        }
        if ended {
            if let Some(button) = elements.try_get_mut(self.play_button.inner) {
                button.element_data_mut().set_accessibility_name("play");
            }
            if let Some(icon) = elements.try_get_as_mut::<TinyVgElement>(self.play_button_icon.inner) {
                icon.set_resource_id(gummy_tree, pending_resources, self.play_icon.clone());
            }
        }
        ended
    }

    fn start_progress_updates(&mut self, pending_animation_updates: &mut Vec<(DynElement, bool)>) {
        self.next_ui_update = Some(Instant::now());
        queue_animation_update(pending_animation_updates, self.element_data.me, false);
        self.request_window_redraw();
    }

    fn stop_progress_updates(&mut self, pending_animation_updates: &mut Vec<(DynElement, bool)>) {
        self.next_ui_update = None;
        queue_animation_update(pending_animation_updates, self.element_data.me, false);
        self.request_window_redraw();
    }

    pub fn set_controls(&mut self, gummy_tree: &mut GummyTree, controls: bool) {
        self.controls = controls;
        self.set_display(gummy_tree, if controls { Display::Flex } else { Display::None });
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
