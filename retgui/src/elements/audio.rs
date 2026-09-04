//! An audio player element with optional controls.

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use maudio::engine::Engine;
use maudio::sound::Sound;
use maudio::sound::notifier::EndNotifier;
use retgui_primitives::brush::Brush;
use retgui_renderer::renderer::Renderer;
use retgui_resource_manager::{ResourceId, ResourceManager};

use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container, push_child_to_element};
use crate::elements::traits::clone_element;
use crate::elements::{Button, DynElement, Element, ElementNode, Elements, Slider, State, Text, TinyVg, scrollable};
use crate::events::EventKind;
use crate::layout::GummyTree;
use crate::style::{AlignItems, Display, Unit};
use crate::text::text_context::TextContext;
use crate::{Color, rgb};

pub(crate) struct SoundData {
    sound: Sound,
    end_notifier: EndNotifier,
}

pub(crate) struct AudioContext {
    engine: Engine,
    pub(crate) sounds: HashSet<DynElement>,
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
            sounds: HashSet::new(),
        }
    }
}

const PLAY: &[u8] = include_bytes!("../../../assets/play.tvg");
const PAUSE: &[u8] = include_bytes!("../../../assets/pause.tvg");
const VOLUME: &[u8] = include_bytes!("../../../assets/volume.tvg");

#[derive(Clone, Copy)]
pub struct Audio {
    pub(crate) inner: DynElement,
}

#[derive(Clone)]
pub(crate) struct AudioNode {
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
}

impl Element for Audio {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl crate::elements::ElementNodeData for AudioNode {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementNode for AudioNode {
    fn deep_clone(&self, elements: &mut Elements) -> DynElement {
        DynElement::new(clone_element::<Self, _>(self, elements, |_, _| None))
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
        elements: &Elements,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        draw_generic_container(self, elements, renderer, resource_manager, text_context, scale_factor);
    }

    fn on_event(&mut self, elements: &mut Elements, event: &mut EventKind, _text_context: &mut TextContext) {
        scrollable::handle_scroll_logic(elements, self, event);
    }
}

impl Audio {
    pub fn new(elements: &mut Elements, path: &Path) -> Self {
        let play_icon = ResourceId::StaticBytes(PLAY);
        let pause_icon = ResourceId::StaticBytes(PAUSE);
        let volume_icon = ResourceId::StaticBytes(VOLUME);

        let play_button = Button::new(elements);
        play_button.set_accessibility_name(elements, "play");
        let play_button_icon = TinyVg::new(elements, play_icon.clone());
        play_button_icon.set_color(elements, Color::WHITE);
        play_button_icon.set_width(elements, Unit::Px(16.0));
        play_button_icon.set_height(elements, Unit::Px(16.0));
        let track = Slider::new(elements, 16.0);
        track.set_width(elements, Unit::Px(200.0));
        track.set_thumb_color(elements, Brush::Color(Color::WHITE));
        let volume_track = Slider::new(elements, 16.0);
        volume_track.set_thumb_color(elements, Brush::Color(Color::WHITE));
        volume_track.set_min(elements, 0.0);
        volume_track.set_max(elements, 100.0);
        volume_track.set_value(elements, 100.0);
        volume_track.set_step(elements, 1.0);
        let duration = Text::new(elements, "");
        duration.set_selectable(elements, false);
        duration.set_line_height(elements, 0.5);
        duration.set_color(elements, Color::WHITE);

        let inner = elements.insert_with(|me, access_tree| {
            Box::new(AudioNode {
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
            })
        });

        {
            let audio = elements.get_as_mut::<AudioNode>(inner);
            audio.set_height(Unit::Px(24.0));
            audio.set_align_items(AlignItems::Center);
            audio.set_background_brush(Brush::Color(rgb(72, 72, 72)));
            audio.set_padding_all(Unit::Px(6.0));
            audio.set_column_gap(Unit::Px(12.0));
        }
        elements.create_layout_node(inner, None);

        play_button.push(elements, play_button_icon);
        play_button.on_click(elements, move |_event, elements| {
            elements.dispatch_mut(inner, |audio, elements| {
                (audio as &mut dyn std::any::Any)
                    .downcast_mut::<AudioNode>()
                    .expect("audio handle changed type")
                    .toggle(elements);
            });
        });
        let play_control = play_button;
        track.on_slider_value_changed(elements, move |event, elements| {
            elements.dispatch_mut(inner, |audio, elements| {
                (audio as &mut dyn std::any::Any)
                    .downcast_mut::<AudioNode>()
                    .expect("audio handle changed type")
                    .set_cursor(elements, event.value as f32);
            });
        });
        let track_control = track;
        volume_track.on_slider_value_changed(elements, move |event, elements| {
            elements.dispatch_mut(inner, |audio, elements| {
                (audio as &mut dyn std::any::Any)
                    .downcast_mut::<AudioNode>()
                    .expect("audio handle changed type")
                    .set_volume(elements, event.value as f32);
            });
        });
        let volume_control = volume_track;
        let volume_icon_element = TinyVg::new(elements, volume_icon);
        volume_icon_element.set_color(elements, Color::WHITE);
        volume_icon_element.set_width(elements, Unit::Px(16.0));
        volume_icon_element.set_height(elements, Unit::Px(16.0));

        for child in [
            play_control.as_dyn_element(),
            track_control.as_dyn_element(),
            duration.as_dyn_element(),
            volume_icon_element.as_dyn_element(),
            volume_control.as_dyn_element(),
        ] {
            push_child_to_element(elements, inner, child);
        }

        elements.dispatch_mut(inner, |audio, elements| {
            (audio as &mut dyn std::any::Any)
                .downcast_mut::<AudioNode>()
                .expect("audio handle changed type")
                .set_sound(elements, path);
        });

        Self { inner }
    }

    pub fn set_controls(&self, elements: &mut Elements, controls: bool) {
        if let Some(audio) = elements.try_get_as_mut::<AudioNode>(self.inner) {
            audio.set_controls(controls);
        }
    }

    pub fn play(&self, elements: &mut Elements) {
        elements.try_dispatch_mut(self.inner, |audio, elements| {
            (audio as &mut dyn std::any::Any)
                .downcast_mut::<AudioNode>()
                .expect("audio handle changed type")
                .play(elements);
        });
    }

    pub fn pause(&self, elements: &mut Elements) {
        elements.try_dispatch_mut(self.inner, |audio, elements| {
            (audio as &mut dyn std::any::Any)
                .downcast_mut::<AudioNode>()
                .expect("audio handle changed type")
                .pause(elements);
        });
    }

    pub fn toggle(&self, elements: &mut Elements) {
        elements.try_dispatch_mut(self.inner, |audio, elements| {
            (audio as &mut dyn std::any::Any)
                .downcast_mut::<AudioNode>()
                .expect("audio handle changed type")
                .toggle(elements);
        });
    }

    pub fn is_playing(&self, elements: &Elements) -> bool {
        elements
            .try_get_as::<AudioNode>(self.inner)
            .is_some_and(|audio| audio.is_playing(elements))
    }
}

impl AudioNode {
    fn toggle(&mut self, elements: &mut Elements) {
        if self.is_playing(elements) {
            self.pause(elements);
        } else {
            self.play(elements);
        }
    }

    fn is_playing(&self, elements: &Elements) -> bool {
        self.sound_data
            .map(|sound| elements.state(sound).sound.is_playing())
            .unwrap_or(false)
    }

    fn set_sound(&mut self, elements: &mut Elements, path: &Path) {
        let (sound, end_notifier, duration, current_time) = elements.with_audio_context(|context, _| {
            let mut sound = context.engine.new_sound_from_file(path).unwrap();
            let end_notifier = sound.set_end_callback().unwrap();
            let duration = sound.length_seconds().unwrap_or_default() as f64;
            let current_time = sound.cursor_seconds().unwrap_or_default() as u32;
            (sound, end_notifier, duration, current_time)
        });

        self.track.set_max(elements, duration);
        self.duration
            .set_text(elements, &format_time(current_time, duration as u32));
        self.sound_data = Some(elements.insert_state(SoundData {
            sound,
            end_notifier,
        }));
        let me = self.element_data.me;
        elements.with_audio_context(|context, _| {
            context.sounds.insert(me);
        });
        let volume = self.volume_track.value(elements) as f32;
        self.set_volume(elements, volume);
    }

    fn play(&self, elements: &mut Elements) {
        self.play_button.set_accessibility_name(elements, "pause");
        self.play_button_icon.set_resource_id(elements, self.pause_icon.clone());
        if let Some(sound_data) = self.sound_data {
            elements
                .state_mut(sound_data)
                .sound
                .play_sound()
                .expect("failed to play sound");
        }
    }

    fn pause(&self, elements: &mut Elements) {
        self.play_button.set_accessibility_name(elements, "play");
        self.play_button_icon.set_resource_id(elements, self.play_icon.clone());
        if let Some(sound_data) = self.sound_data {
            elements
                .state_mut(sound_data)
                .sound
                .stop_sound()
                .expect("failed to pause sound");
        }
    }

    fn set_cursor(&self, elements: &mut Elements, value: f32) {
        if let Some(sound_data) = self.sound_data {
            elements.state_mut(sound_data).sound.seek_to_second(value).unwrap();
        }
    }

    fn set_volume(&self, elements: &mut Elements, value: f32) {
        if let Some(sound_data) = self.sound_data {
            elements.state_mut(sound_data).sound.set_volume(value / 100.0);
        }
    }

    pub(crate) fn update(&self, elements: &mut Elements) {
        let Some(sound_data) = self.sound_data else {
            return;
        };
        let mut ended = false;
        let (current_time, total_time) = {
            let sound_data = elements.state_mut(sound_data);
            let current_time = sound_data.sound.cursor_seconds().unwrap_or_default() as f64;
            let total_time = sound_data.sound.length_seconds().unwrap_or_default() as u32;
            sound_data.end_notifier.take_with(|| ended = true);
            (current_time, total_time)
        };

        if self.track.value(elements) != current_time {
            self.track.set_value(elements, current_time);
            self.duration
                .set_text(elements, &format_time(current_time as u32, total_time));
        }
        if ended {
            self.play_button_icon.set_resource_id(elements, self.play_icon.clone());
        }
    }

    pub fn set_controls(&mut self, controls: bool) {
        self.controls = controls;
        self.set_display(if controls { Display::Flex } else { Display::None });
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
