use craft::style::{AlignItems, JustifyContent};
use craft::components::{Component, ComponentId, ComponentSpecification, UpdateResult};
use craft::elements::{Container, ElementStyles, Text};
use craft::events::CraftMessage::Initialized;
use craft::events::Event;
use craft::events::Message::CraftMessage;
use craft::style::{Display, FlexDirection, Weight};
use craft::{Color, WindowContext};
use rodio::OutputStream;
use std::io::BufReader;
use std::io::Cursor;
use std::thread::spawn;
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Default)]
pub(crate) struct Mp3Player {
    sender: Option<Sender<Mp3PlayerMessage>>,
    player_state: Mp3PlayerMessage,
}

#[derive(Default, Debug)]
#[derive(PartialEq)]
enum Mp3PlayerMessage {
    #[default]
    Play,
    Pause,
}

impl Component for Mp3Player {
    type Props = ();

    fn view_with_no_global_state(state: &Self, _props: &Self::Props, _children: Vec<ComponentSpecification>, _id: ComponentId, _window_context: &WindowContext) -> ComponentSpecification {
        
        
        let play_pause_button = if state.player_state == Mp3PlayerMessage::Play {
            Text::new("Play")
                .id("play_audio")
                .padding("10px", "16px", "10px", "16px")
                .background(Color::from_rgb8(76, 175, 80))
                .color(Color::WHITE)
                .border_radius(8.0, 8.0, 8.0, 8.0)
        } else {
            Text::new("Pause")
                .id("pause_audio")
                .padding("10px", "16px", "10px", "16px")
                .background(Color::from_rgb8(244, 67, 54))
                .color(Color::WHITE)
                .border_radius(8.0, 8.0, 8.0, 8.0)
        };
        
        Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .padding("24px", "24px", "24px", "24px")
            .background(Color::WHITE)
            .border_radius(12.0, 12.0, 12.0, 12.0)
            .max_width("400px")
            .max_height("200px")
            .gap("16px")
            .push(
                Text::new("🎵 Now Playing")
                    .font_size(16.0)
                    .font_weight(Weight::SEMIBOLD)
                    .color(Color::from_rgb8(50, 50, 50))
            )
            .push(
                Text::new("No More Magic")
                    .font_size(20.0)
                    .font_weight(Weight::BOLD)
                    .color(Color::from_rgb8(30, 30, 30))
            )
            .push(
                play_pause_button
            )
            .component()
    }

    fn update_with_no_global_state(
        state: &mut Self,
        _props: &Self::Props,
        event: Event,
        _window_context: &mut WindowContext,
    ) -> UpdateResult {
        
        if let CraftMessage(Initialized) = event.message {
            let (sender, receiver) = tokio::sync::mpsc::channel::<Mp3PlayerMessage>(100);
            state.sender = Some(sender.clone());
            spawn(|| run_mp3_player_thread(receiver, include_bytes!("./No More Magic.mp3")));
        }

        if event.message.clicked() {
            if let Some(sender) = &state.sender {
                let audio_message = match event.current_target.as_deref() {
                    Some("play_audio") =>  {
                        // Flip the state.
                        state.player_state = Mp3PlayerMessage::Pause;
                        Some(Mp3PlayerMessage::Play)
                    },
                    Some("pause_audio") =>  {
                        // Flip the state.
                        state.player_state = Mp3PlayerMessage::Play;
                        Some(Mp3PlayerMessage::Pause)
                    },
                    _ => None,
                };

                if let Some(message) = audio_message {
                    return send_mp3_command(sender.clone(), message);
                }
            }
        }

        UpdateResult::new()
    }
}

fn run_mp3_player_thread(mut receiver: Receiver<Mp3PlayerMessage>, mp3_bytes: &[u8]) {
    let mp3_bytes = mp3_bytes.to_vec();
    let file_cursor = Cursor::new(mp3_bytes);
    let (_stream, handle) = OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();
    sink.append(rodio::Decoder::new(BufReader::new(file_cursor)).unwrap());
    sink.pause();

    while let Some(message) = receiver.blocking_recv() {
        match message {
            Mp3PlayerMessage::Play => sink.play(),
            Mp3PlayerMessage::Pause => sink.pause(),
        }
    }
}

fn send_mp3_command(sender: Sender<Mp3PlayerMessage>, message: Mp3PlayerMessage) -> UpdateResult {
    let spawn_thread = async move {
        sender.send(message).await.unwrap();
        UpdateResult::async_result(())
    };

    UpdateResult::new().future(spawn_thread)
}