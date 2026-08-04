use crate::Event as BaseEvent;
use crate::PhyPoint;
use iced_runtime::core::Event as IcedEvent;
use iced_runtime::core::Point;
use iced_runtime::core::mouse::Button as IcedMouseButton;
use iced_runtime::core::mouse::Event as IcedMouseEvent;
use iced_runtime::core::window::Event as IcedWindowEvent;
use iced_runtime::keyboard::Event as IcedKeyEvent;
use iced_runtime::keyboard::Modifiers as IcedModifiers;
use keyboard_types::Modifiers as BaseviewModifiers;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub fn baseview_to_iced_events(
    event: BaseEvent, iced_events: &mut Vec<(iced_core::window::Id, IcedEvent)>,
    iced_modifiers: &mut IcedModifiers, ignore_non_modifier_keys: bool,
    window_id: iced_core::window::Id,
) {
    match event {
        BaseEvent::Mouse(mouse_event) => match mouse_event {
            crate::MouseEvent::CursorMoved { position, modifiers } => {
                if let Some(event) = update_modifiers(iced_modifiers, modifiers) {
                    iced_events.push((window_id, event));
                }
                iced_events.push((
                    window_id,
                    IcedEvent::Mouse(IcedMouseEvent::CursorMoved {
                        position: Point::new(position.x as f32, position.y as f32),
                    }),
                ));
            }
            crate::MouseEvent::ButtonPressed { button, modifiers } => {
                if let Some(event) = update_modifiers(iced_modifiers, modifiers) {
                    iced_events.push((window_id, event));
                }
                iced_events.push((
                    window_id,
                    IcedEvent::Mouse(IcedMouseEvent::ButtonPressed(baseview_mouse_button_to_iced(
                        button,
                    ))),
                ));
            }
            crate::MouseEvent::ButtonReleased { button, modifiers } => {
                if let Some(event) = update_modifiers(iced_modifiers, modifiers) {
                    iced_events.push((window_id, event));
                }
                iced_events.push((
                    window_id,
                    IcedEvent::Mouse(IcedMouseEvent::ButtonReleased(
                        baseview_mouse_button_to_iced(button),
                    )),
                ));
            }
            crate::MouseEvent::WheelScrolled { delta, modifiers } => match delta {
                crate::ScrollDelta::Lines { x, y } => {
                    if let Some(event) = update_modifiers(iced_modifiers, modifiers) {
                        iced_events.push((window_id, event));
                    }
                    iced_events.push((
                        window_id,
                        IcedEvent::Mouse(IcedMouseEvent::WheelScrolled {
                            delta: iced_runtime::core::mouse::ScrollDelta::Lines { x, y },
                        }),
                    ));
                }
                crate::ScrollDelta::Pixels { x, y } => {
                    if let Some(event) = update_modifiers(iced_modifiers, modifiers) {
                        iced_events.push((window_id, event));
                    }
                    iced_events.push((
                        window_id,
                        IcedEvent::Mouse(IcedMouseEvent::WheelScrolled {
                            delta: iced_runtime::core::mouse::ScrollDelta::Pixels { x, y },
                        }),
                    ));
                }
            },
            crate::MouseEvent::CursorLeft => {
                iced_events.push((window_id, IcedEvent::Mouse(IcedMouseEvent::CursorLeft)));
            }
            crate::MouseEvent::CursorEntered => {
                iced_events.push((window_id, IcedEvent::Mouse(IcedMouseEvent::CursorEntered)));
            }
            _ => {}
        },

        BaseEvent::Keyboard(event) => {
            if let Some(event) = update_modifiers(iced_modifiers, event.modifiers) {
                iced_events.push((window_id, event));
            }

            if ignore_non_modifier_keys {
                return;
            }

            let is_down = match event.state {
                keyboard_types::KeyState::Down => true,
                keyboard_types::KeyState::Up => false,
            };

            let key = baseview_to_iced_key(event.key);
            let location = baseview_key_location_to_iced(event.location);

            let physical_key = if let Some(code) = baseview_to_iced_keycode(event.code) {
                iced_runtime::core::keyboard::key::Physical::Code(code)
            } else {
                iced_runtime::core::keyboard::key::Physical::Unidentified(
                    iced_runtime::core::keyboard::key::NativeCode::Unidentified,
                )
            };

            if is_down {
                let text = if let iced_runtime::core::keyboard::Key::Character(s) = &key {
                    Some(s.clone())
                } else {
                    None
                };

                iced_events.push((
                    window_id,
                    IcedEvent::Keyboard(IcedKeyEvent::KeyPressed {
                        key: key.clone(),
                        modified_key: key,
                        physical_key,
                        modifiers: *iced_modifiers,
                        location,
                        text,
                        repeat: event.repeat,
                    }),
                ));
            } else {
                iced_events.push((
                    window_id,
                    IcedEvent::Keyboard(IcedKeyEvent::KeyReleased {
                        key: key.clone(),
                        location,
                        modifiers: *iced_modifiers,
                        modified_key: key,
                        physical_key,
                    }),
                ));
            }
        }

        BaseEvent::Window(window_event) => match window_event {
            crate::WindowEvent::Resized(window_info) => {
                iced_events.push((
                    window_id,
                    IcedEvent::Window(IcedWindowEvent::Resized(iced_runtime::core::Size {
                        width: window_info.logical_size().width as f32,
                        height: window_info.logical_size().height as f32,
                    })),
                ));
            }
            crate::WindowEvent::Unfocused => {
                *iced_modifiers = IcedModifiers::empty();
                iced_events.push((window_id, IcedEvent::Window(IcedWindowEvent::Unfocused)));
            }
            crate::WindowEvent::Focused => {
                *iced_modifiers = IcedModifiers::empty();
                iced_events.push((window_id, IcedEvent::Window(IcedWindowEvent::Focused)));
            }
            _ => {}
        },
    }
}

fn update_modifiers(
    iced_modifiers: &mut IcedModifiers, baseview_modifiers: BaseviewModifiers,
) -> Option<IcedEvent> {
    let mut new = IcedModifiers::default();

    new.set(IcedModifiers::ALT, baseview_modifiers.contains(BaseviewModifiers::ALT));
    new.set(IcedModifiers::CTRL, baseview_modifiers.contains(BaseviewModifiers::CONTROL));
    new.set(IcedModifiers::SHIFT, baseview_modifiers.contains(BaseviewModifiers::SHIFT));
    new.set(IcedModifiers::LOGO, baseview_modifiers.contains(BaseviewModifiers::META));

    if *iced_modifiers != new {
        *iced_modifiers = new;

        Some(IcedEvent::Keyboard(iced_runtime::core::keyboard::Event::ModifiersChanged(
            *iced_modifiers,
        )))
    } else {
        None
    }
}

fn baseview_mouse_button_to_iced(id: crate::MouseButton) -> IcedMouseButton {
    use crate::MouseButton;

    match id {
        MouseButton::Left => IcedMouseButton::Left,
        MouseButton::Middle => IcedMouseButton::Middle,
        MouseButton::Right => IcedMouseButton::Right,
        MouseButton::Back => IcedMouseButton::Other(6),
        MouseButton::Forward => IcedMouseButton::Other(7),
        MouseButton::Other(other_id) => IcedMouseButton::Other(other_id as u16),
    }
}

pub fn cursor_position(position: PhyPoint, scale_factor: f32) -> Point {
    Point::new(
        (f64::from(position.x) * scale_factor as f64) as f32,
        (f64::from(position.y) * scale_factor as f64) as f32,
    )
}

fn baseview_key_location_to_iced(
    location: keyboard_types::Location,
) -> iced_runtime::core::keyboard::Location {
    use iced_runtime::core::keyboard::Location as ILocation;
    use keyboard_types::Location as KLocation;

    match location {
        KLocation::Standard => ILocation::Standard,
        KLocation::Left => ILocation::Left,
        KLocation::Right => ILocation::Right,
        KLocation::Numpad => ILocation::Numpad,
    }
}

fn baseview_to_iced_key(key: keyboard_types::Key) -> iced_runtime::core::keyboard::Key {
    use iced_runtime::core::keyboard::Key as IKey;
    use iced_runtime::core::keyboard::key::Named as IN;
    use keyboard_types::Key as KKey;
    use keyboard_types::NamedKey as KN;

    match key {
        KKey::Character(s) => IKey::Character(s.into()),

        KKey::Named(named) => match named {
            KN::Alt => IKey::Named(IN::Alt),
            KN::AltGraph => IKey::Named(IN::AltGraph),
            KN::CapsLock => IKey::Named(IN::CapsLock),
            KN::Control => IKey::Named(IN::Control),
            KN::Fn => IKey::Named(IN::Fn),
            KN::FnLock => IKey::Named(IN::FnLock),
            KN::Meta => IKey::Named(IN::Meta),
            KN::NumLock => IKey::Named(IN::NumLock),
            KN::ScrollLock => IKey::Named(IN::ScrollLock),
            KN::Shift => IKey::Named(IN::Shift),
            KN::Symbol => IKey::Named(IN::Symbol),
            KN::SymbolLock => IKey::Named(IN::SymbolLock),
            #[expect(deprecated)]
            KN::Hyper => IKey::Named(IN::Meta),
            #[expect(deprecated)]
            KN::Super => IKey::Named(IN::Meta),
            KN::Enter => IKey::Named(IN::Enter),
            KN::Tab => IKey::Named(IN::Tab),
            KN::ArrowDown => IKey::Named(IN::ArrowDown),
            KN::ArrowLeft => IKey::Named(IN::ArrowLeft),
            KN::ArrowRight => IKey::Named(IN::ArrowRight),
            KN::ArrowUp => IKey::Named(IN::ArrowUp),
            KN::End => IKey::Named(IN::End),
            KN::Home => IKey::Named(IN::Home),
            KN::PageDown => IKey::Named(IN::PageDown),
            KN::PageUp => IKey::Named(IN::PageUp),
            KN::Backspace => IKey::Named(IN::Backspace),
            KN::Clear => IKey::Named(IN::Clear),
            KN::Copy => IKey::Named(IN::Copy),
            KN::CrSel => IKey::Named(IN::CrSel),
            KN::Cut => IKey::Named(IN::Cut),
            KN::Delete => IKey::Named(IN::Delete),
            KN::EraseEof => IKey::Named(IN::EraseEof),
            KN::ExSel => IKey::Named(IN::ExSel),
            KN::Insert => IKey::Named(IN::Insert),
            KN::Paste => IKey::Named(IN::Paste),
            KN::Redo => IKey::Named(IN::Redo),
            KN::Undo => IKey::Named(IN::Undo),
            KN::Accept => IKey::Named(IN::Accept),
            KN::Again => IKey::Named(IN::Again),
            KN::Attn => IKey::Named(IN::Attn),
            KN::Cancel => IKey::Named(IN::Cancel),
            KN::ContextMenu => IKey::Named(IN::ContextMenu),
            KN::Escape => IKey::Named(IN::Escape),
            KN::Execute => IKey::Named(IN::Execute),
            KN::Find => IKey::Named(IN::Find),
            KN::Help => IKey::Named(IN::Help),
            KN::Pause => IKey::Named(IN::Pause),
            KN::Play => IKey::Named(IN::Play),
            KN::Props => IKey::Named(IN::Props),
            KN::Select => IKey::Named(IN::Select),
            KN::ZoomIn => IKey::Named(IN::ZoomIn),
            KN::ZoomOut => IKey::Named(IN::ZoomOut),
            KN::BrightnessDown => IKey::Named(IN::BrightnessDown),
            KN::BrightnessUp => IKey::Named(IN::BrightnessUp),
            KN::Eject => IKey::Named(IN::Eject),
            KN::LogOff => IKey::Named(IN::LogOff),
            KN::Power => IKey::Named(IN::Power),
            KN::PowerOff => IKey::Named(IN::PowerOff),
            KN::PrintScreen => IKey::Named(IN::PrintScreen),
            KN::Hibernate => IKey::Named(IN::Hibernate),
            KN::Standby => IKey::Named(IN::Standby),
            KN::WakeUp => IKey::Named(IN::WakeUp),
            KN::AllCandidates => IKey::Named(IN::AllCandidates),
            KN::Alphanumeric => IKey::Named(IN::Alphanumeric),
            KN::CodeInput => IKey::Named(IN::CodeInput),
            KN::Compose => IKey::Named(IN::Compose),
            KN::Convert => IKey::Named(IN::Convert),
            KN::FinalMode => IKey::Named(IN::FinalMode),
            KN::GroupFirst => IKey::Named(IN::GroupFirst),
            KN::GroupLast => IKey::Named(IN::GroupLast),
            KN::GroupNext => IKey::Named(IN::GroupNext),
            KN::GroupPrevious => IKey::Named(IN::GroupPrevious),
            KN::ModeChange => IKey::Named(IN::ModeChange),
            KN::NextCandidate => IKey::Named(IN::NextCandidate),
            KN::NonConvert => IKey::Named(IN::NonConvert),
            KN::PreviousCandidate => IKey::Named(IN::PreviousCandidate),
            KN::Process => IKey::Named(IN::Process),
            KN::SingleCandidate => IKey::Named(IN::SingleCandidate),
            KN::HangulMode => IKey::Named(IN::HangulMode),
            KN::HanjaMode => IKey::Named(IN::HanjaMode),
            KN::JunjaMode => IKey::Named(IN::JunjaMode),
            KN::Eisu => IKey::Named(IN::Eisu),
            KN::Hankaku => IKey::Named(IN::Hankaku),
            KN::Hiragana => IKey::Named(IN::Hiragana),
            KN::HiraganaKatakana => IKey::Named(IN::HiraganaKatakana),
            KN::KanaMode => IKey::Named(IN::KanaMode),
            KN::KanjiMode => IKey::Named(IN::KanjiMode),
            KN::Katakana => IKey::Named(IN::Katakana),
            KN::Romaji => IKey::Named(IN::Romaji),
            KN::Zenkaku => IKey::Named(IN::Zenkaku),
            KN::ZenkakuHankaku => IKey::Named(IN::ZenkakuHankaku),
            KN::F1 => IKey::Named(IN::F1),
            KN::F2 => IKey::Named(IN::F2),
            KN::F3 => IKey::Named(IN::F3),
            KN::F4 => IKey::Named(IN::F4),
            KN::F5 => IKey::Named(IN::F5),
            KN::F6 => IKey::Named(IN::F6),
            KN::F7 => IKey::Named(IN::F7),
            KN::F8 => IKey::Named(IN::F8),
            KN::F9 => IKey::Named(IN::F9),
            KN::F10 => IKey::Named(IN::F10),
            KN::F11 => IKey::Named(IN::F11),
            KN::F12 => IKey::Named(IN::F12),
            KN::Soft1 => IKey::Named(IN::Soft1),
            KN::Soft2 => IKey::Named(IN::Soft2),
            KN::Soft3 => IKey::Named(IN::Soft3),
            KN::Soft4 => IKey::Named(IN::Soft4),
            KN::ChannelDown => IKey::Named(IN::ChannelDown),
            KN::ChannelUp => IKey::Named(IN::ChannelUp),
            KN::Close => IKey::Named(IN::Close),
            KN::MailForward => IKey::Named(IN::MailForward),
            KN::MailReply => IKey::Named(IN::MailReply),
            KN::MailSend => IKey::Named(IN::MailSend),
            KN::MediaClose => IKey::Named(IN::MediaClose),
            KN::MediaFastForward => IKey::Named(IN::MediaFastForward),
            KN::MediaPause => IKey::Named(IN::MediaPause),
            KN::MediaPlay => IKey::Named(IN::MediaPlay),
            KN::MediaPlayPause => IKey::Named(IN::MediaPlayPause),
            KN::MediaRecord => IKey::Named(IN::MediaRecord),
            KN::MediaRewind => IKey::Named(IN::MediaRewind),
            KN::MediaStop => IKey::Named(IN::MediaStop),
            KN::MediaTrackNext => IKey::Named(IN::MediaTrackNext),
            KN::MediaTrackPrevious => IKey::Named(IN::MediaTrackPrevious),
            KN::New => IKey::Named(IN::New),
            KN::Open => IKey::Named(IN::Open),
            KN::Print => IKey::Named(IN::Print),
            KN::Save => IKey::Named(IN::Save),
            KN::SpellCheck => IKey::Named(IN::SpellCheck),
            KN::Key11 => IKey::Named(IN::Key11),
            KN::Key12 => IKey::Named(IN::Key12),
            KN::AudioBalanceLeft => IKey::Named(IN::AudioBalanceLeft),
            KN::AudioBalanceRight => IKey::Named(IN::AudioBalanceRight),
            KN::AudioBassBoostDown => IKey::Named(IN::AudioBassBoostDown),
            KN::AudioBassBoostToggle => IKey::Named(IN::AudioBassBoostToggle),
            KN::AudioBassBoostUp => IKey::Named(IN::AudioBassBoostUp),
            KN::AudioFaderFront => IKey::Named(IN::AudioFaderFront),
            KN::AudioFaderRear => IKey::Named(IN::AudioFaderRear),
            KN::AudioSurroundModeNext => IKey::Named(IN::AudioSurroundModeNext),
            KN::AudioTrebleDown => IKey::Named(IN::AudioTrebleDown),
            KN::AudioTrebleUp => IKey::Named(IN::AudioTrebleUp),
            KN::AudioVolumeDown => IKey::Named(IN::AudioVolumeDown),
            KN::AudioVolumeUp => IKey::Named(IN::AudioVolumeUp),
            KN::AudioVolumeMute => IKey::Named(IN::AudioVolumeMute),
            KN::MicrophoneToggle => IKey::Named(IN::MicrophoneToggle),
            KN::MicrophoneVolumeDown => IKey::Named(IN::MicrophoneVolumeDown),
            KN::MicrophoneVolumeUp => IKey::Named(IN::MicrophoneVolumeUp),
            KN::MicrophoneVolumeMute => IKey::Named(IN::MicrophoneVolumeMute),
            KN::SpeechCorrectionList => IKey::Named(IN::SpeechCorrectionList),
            KN::SpeechInputToggle => IKey::Named(IN::SpeechInputToggle),
            KN::LaunchApplication1 => IKey::Named(IN::LaunchApplication1),
            KN::LaunchApplication2 => IKey::Named(IN::LaunchApplication2),
            KN::LaunchCalendar => IKey::Named(IN::LaunchCalendar),
            KN::LaunchContacts => IKey::Named(IN::LaunchContacts),
            KN::LaunchMail => IKey::Named(IN::LaunchMail),
            KN::LaunchMediaPlayer => IKey::Named(IN::LaunchMediaPlayer),
            KN::LaunchMusicPlayer => IKey::Named(IN::LaunchMusicPlayer),
            KN::LaunchPhone => IKey::Named(IN::LaunchPhone),
            KN::LaunchScreenSaver => IKey::Named(IN::LaunchScreenSaver),
            KN::LaunchSpreadsheet => IKey::Named(IN::LaunchSpreadsheet),
            KN::LaunchWebBrowser => IKey::Named(IN::LaunchWebBrowser),
            KN::LaunchWebCam => IKey::Named(IN::LaunchWebCam),
            KN::LaunchWordProcessor => IKey::Named(IN::LaunchWordProcessor),
            KN::BrowserBack => IKey::Named(IN::BrowserBack),
            KN::BrowserFavorites => IKey::Named(IN::BrowserFavorites),
            KN::BrowserForward => IKey::Named(IN::BrowserForward),
            KN::BrowserHome => IKey::Named(IN::BrowserHome),
            KN::BrowserRefresh => IKey::Named(IN::BrowserRefresh),
            KN::BrowserSearch => IKey::Named(IN::BrowserSearch),
            KN::BrowserStop => IKey::Named(IN::BrowserStop),
            KN::AppSwitch => IKey::Named(IN::AppSwitch),
            KN::Call => IKey::Named(IN::Call),
            KN::Camera => IKey::Named(IN::Camera),
            KN::CameraFocus => IKey::Named(IN::CameraFocus),
            KN::EndCall => IKey::Named(IN::EndCall),
            KN::GoBack => IKey::Named(IN::GoBack),
            KN::GoHome => IKey::Named(IN::GoHome),
            KN::HeadsetHook => IKey::Named(IN::HeadsetHook),
            KN::LastNumberRedial => IKey::Named(IN::LastNumberRedial),
            KN::Notification => IKey::Named(IN::Notification),
            KN::MannerMode => IKey::Named(IN::MannerMode),
            KN::VoiceDial => IKey::Named(IN::VoiceDial),
            KN::TV => IKey::Named(IN::TV),
            KN::TV3DMode => IKey::Named(IN::TV3DMode),
            KN::TVAntennaCable => IKey::Named(IN::TVAntennaCable),
            KN::TVAudioDescription => IKey::Named(IN::TVAudioDescription),
            KN::TVAudioDescriptionMixDown => IKey::Named(IN::TVAudioDescriptionMixDown),
            KN::TVAudioDescriptionMixUp => IKey::Named(IN::TVAudioDescriptionMixUp),
            KN::TVContentsMenu => IKey::Named(IN::TVContentsMenu),
            KN::TVDataService => IKey::Named(IN::TVDataService),
            KN::TVInput => IKey::Named(IN::TVInput),
            KN::TVInputComponent1 => IKey::Named(IN::TVInputComponent1),
            KN::TVInputComponent2 => IKey::Named(IN::TVInputComponent2),
            KN::TVInputComposite1 => IKey::Named(IN::TVInputComposite1),
            KN::TVInputComposite2 => IKey::Named(IN::TVInputComposite2),
            KN::TVInputHDMI1 => IKey::Named(IN::TVInputHDMI1),
            KN::TVInputHDMI2 => IKey::Named(IN::TVInputHDMI2),
            KN::TVInputHDMI3 => IKey::Named(IN::TVInputHDMI3),
            KN::TVInputHDMI4 => IKey::Named(IN::TVInputHDMI4),
            KN::TVInputVGA1 => IKey::Named(IN::TVInputVGA1),
            KN::TVMediaContext => IKey::Named(IN::TVMediaContext),
            KN::TVNetwork => IKey::Named(IN::TVNetwork),
            KN::TVNumberEntry => IKey::Named(IN::TVNumberEntry),
            KN::TVPower => IKey::Named(IN::TVPower),
            KN::TVRadioService => IKey::Named(IN::TVRadioService),
            KN::TVSatellite => IKey::Named(IN::TVSatellite),
            KN::TVSatelliteBS => IKey::Named(IN::TVSatelliteBS),
            KN::TVSatelliteCS => IKey::Named(IN::TVSatelliteCS),
            KN::TVSatelliteToggle => IKey::Named(IN::TVSatelliteToggle),
            KN::TVTerrestrialAnalog => IKey::Named(IN::TVTerrestrialAnalog),
            KN::TVTerrestrialDigital => IKey::Named(IN::TVTerrestrialDigital),
            KN::TVTimer => IKey::Named(IN::TVTimer),
            KN::AVRInput => IKey::Named(IN::AVRInput),
            KN::AVRPower => IKey::Named(IN::AVRPower),
            KN::ColorF0Red => IKey::Named(IN::ColorF0Red),
            KN::ColorF1Green => IKey::Named(IN::ColorF1Green),
            KN::ColorF2Yellow => IKey::Named(IN::ColorF2Yellow),
            KN::ColorF3Blue => IKey::Named(IN::ColorF3Blue),
            KN::ColorF4Grey => IKey::Named(IN::ColorF4Grey),
            KN::ColorF5Brown => IKey::Named(IN::ColorF5Brown),
            KN::ClosedCaptionToggle => IKey::Named(IN::ClosedCaptionToggle),
            KN::Dimmer => IKey::Named(IN::Dimmer),
            KN::DisplaySwap => IKey::Named(IN::DisplaySwap),
            KN::DVR => IKey::Named(IN::DVR),
            KN::Exit => IKey::Named(IN::Exit),
            KN::FavoriteClear0 => IKey::Named(IN::FavoriteClear0),
            KN::FavoriteClear1 => IKey::Named(IN::FavoriteClear1),
            KN::FavoriteClear2 => IKey::Named(IN::FavoriteClear2),
            KN::FavoriteClear3 => IKey::Named(IN::FavoriteClear3),
            KN::FavoriteRecall0 => IKey::Named(IN::FavoriteRecall0),
            KN::FavoriteRecall1 => IKey::Named(IN::FavoriteRecall1),
            KN::FavoriteRecall2 => IKey::Named(IN::FavoriteRecall2),
            KN::FavoriteRecall3 => IKey::Named(IN::FavoriteRecall3),
            KN::FavoriteStore0 => IKey::Named(IN::FavoriteStore0),
            KN::FavoriteStore1 => IKey::Named(IN::FavoriteStore1),
            KN::FavoriteStore2 => IKey::Named(IN::FavoriteStore2),
            KN::FavoriteStore3 => IKey::Named(IN::FavoriteStore3),
            KN::Guide => IKey::Named(IN::Guide),
            KN::GuideNextDay => IKey::Named(IN::GuideNextDay),
            KN::GuidePreviousDay => IKey::Named(IN::GuidePreviousDay),
            KN::Info => IKey::Named(IN::Info),
            KN::InstantReplay => IKey::Named(IN::InstantReplay),
            KN::Link => IKey::Named(IN::Link),
            KN::ListProgram => IKey::Named(IN::ListProgram),
            KN::LiveContent => IKey::Named(IN::LiveContent),
            KN::Lock => IKey::Named(IN::Lock),
            KN::MediaApps => IKey::Named(IN::MediaApps),
            KN::MediaAudioTrack => IKey::Named(IN::MediaAudioTrack),
            KN::MediaLast => IKey::Named(IN::MediaLast),
            KN::MediaSkipBackward => IKey::Named(IN::MediaSkipBackward),
            KN::MediaSkipForward => IKey::Named(IN::MediaSkipForward),
            KN::MediaStepBackward => IKey::Named(IN::MediaStepBackward),
            KN::MediaStepForward => IKey::Named(IN::MediaStepForward),
            KN::MediaTopMenu => IKey::Named(IN::MediaTopMenu),
            KN::NavigateIn => IKey::Named(IN::NavigateIn),
            KN::NavigateNext => IKey::Named(IN::NavigateNext),
            KN::NavigateOut => IKey::Named(IN::NavigateOut),
            KN::NavigatePrevious => IKey::Named(IN::NavigatePrevious),
            KN::NextFavoriteChannel => IKey::Named(IN::NextFavoriteChannel),
            KN::NextUserProfile => IKey::Named(IN::NextUserProfile),
            KN::OnDemand => IKey::Named(IN::OnDemand),
            KN::Pairing => IKey::Named(IN::Pairing),
            KN::PinPMove => IKey::Named(IN::PinPMove),
            KN::PinPToggle => IKey::Named(IN::PinPToggle),
            KN::PinPUp => IKey::Named(IN::PinPUp),
            KN::PlaySpeedDown => IKey::Named(IN::PlaySpeedDown),
            KN::PlaySpeedReset => IKey::Named(IN::PlaySpeedReset),
            KN::PlaySpeedUp => IKey::Named(IN::PlaySpeedUp),
            KN::RandomToggle => IKey::Named(IN::RandomToggle),
            KN::RcLowBattery => IKey::Named(IN::RcLowBattery),
            KN::RecordSpeedNext => IKey::Named(IN::RecordSpeedNext),
            KN::RfBypass => IKey::Named(IN::RfBypass),
            KN::ScanChannelsToggle => IKey::Named(IN::ScanChannelsToggle),
            KN::ScreenModeNext => IKey::Named(IN::ScreenModeNext),
            KN::Settings => IKey::Named(IN::Settings),
            KN::SplitScreenToggle => IKey::Named(IN::SplitScreenToggle),
            KN::STBInput => IKey::Named(IN::STBInput),
            KN::STBPower => IKey::Named(IN::STBPower),
            KN::Subtitle => IKey::Named(IN::Subtitle),
            KN::Teletext => IKey::Named(IN::Teletext),
            KN::VideoModeNext => IKey::Named(IN::VideoModeNext),
            KN::Wink => IKey::Named(IN::Wink),
            KN::ZoomToggle => IKey::Named(IN::ZoomToggle),
            KN::F13 => IKey::Named(IN::F13),
            KN::F14 => IKey::Named(IN::F14),
            KN::F15 => IKey::Named(IN::F15),
            KN::F16 => IKey::Named(IN::F16),
            KN::F17 => IKey::Named(IN::F17),
            KN::F18 => IKey::Named(IN::F18),
            KN::F19 => IKey::Named(IN::F19),
            KN::F20 => IKey::Named(IN::F20),
            KN::F21 => IKey::Named(IN::F21),
            KN::F22 => IKey::Named(IN::F22),
            KN::F23 => IKey::Named(IN::F23),
            KN::F24 => IKey::Named(IN::F24),
            _ => IKey::Unidentified,
        },
    }
}

fn baseview_to_iced_keycode(
    code: keyboard_types::Code,
) -> Option<iced_runtime::core::keyboard::key::Code> {
    use iced_runtime::core::keyboard::key::Code as ICode;
    use keyboard_types::Code as KCode;

    match code {
        KCode::Digit1 => Some(ICode::Numpad1),
        KCode::Digit2 => Some(ICode::Numpad2),
        KCode::Digit3 => Some(ICode::Numpad3),
        KCode::Digit4 => Some(ICode::Numpad4),
        KCode::Digit5 => Some(ICode::Numpad5),
        KCode::Digit6 => Some(ICode::Numpad6),
        KCode::Digit7 => Some(ICode::Numpad7),
        KCode::Digit8 => Some(ICode::Numpad8),
        KCode::Digit9 => Some(ICode::Numpad9),
        KCode::Digit0 => Some(ICode::Numpad0),

        KCode::KeyA => Some(ICode::KeyA),
        KCode::KeyB => Some(ICode::KeyB),
        KCode::KeyC => Some(ICode::KeyC),
        KCode::KeyD => Some(ICode::KeyD),
        KCode::KeyE => Some(ICode::KeyE),
        KCode::KeyF => Some(ICode::KeyF),
        KCode::KeyG => Some(ICode::KeyG),
        KCode::KeyH => Some(ICode::KeyH),
        KCode::KeyI => Some(ICode::KeyI),
        KCode::KeyJ => Some(ICode::KeyJ),
        KCode::KeyK => Some(ICode::KeyK),
        KCode::KeyL => Some(ICode::KeyL),
        KCode::KeyM => Some(ICode::KeyM),
        KCode::KeyN => Some(ICode::KeyN),
        KCode::KeyO => Some(ICode::KeyO),
        KCode::KeyP => Some(ICode::KeyP),
        KCode::KeyQ => Some(ICode::KeyQ),
        KCode::KeyR => Some(ICode::KeyR),
        KCode::KeyS => Some(ICode::KeyS),
        KCode::KeyT => Some(ICode::KeyT),
        KCode::KeyU => Some(ICode::KeyU),
        KCode::KeyV => Some(ICode::KeyV),
        KCode::KeyW => Some(ICode::KeyW),
        KCode::KeyX => Some(ICode::KeyX),
        KCode::KeyY => Some(ICode::KeyY),
        KCode::KeyZ => Some(ICode::KeyZ),

        KCode::Escape => Some(ICode::Escape),

        KCode::F1 => Some(ICode::F1),
        KCode::F2 => Some(ICode::F2),
        KCode::F3 => Some(ICode::F3),
        KCode::F4 => Some(ICode::F4),
        KCode::F5 => Some(ICode::F5),
        KCode::F6 => Some(ICode::F6),
        KCode::F7 => Some(ICode::F7),
        KCode::F8 => Some(ICode::F8),
        KCode::F9 => Some(ICode::F9),
        KCode::F10 => Some(ICode::F10),
        KCode::F11 => Some(ICode::F11),
        KCode::F12 => Some(ICode::F12),

        KCode::PrintScreen => Some(ICode::PrintScreen),
        KCode::ScrollLock => Some(ICode::ScrollLock),
        KCode::Pause => Some(ICode::Pause),

        KCode::Insert => Some(ICode::Insert),
        KCode::Home => Some(ICode::Home),
        KCode::Delete => Some(ICode::Delete),
        KCode::End => Some(ICode::End),
        KCode::PageDown => Some(ICode::PageDown),
        KCode::PageUp => Some(ICode::PageUp),

        KCode::ArrowLeft => Some(ICode::ArrowLeft),
        KCode::ArrowUp => Some(ICode::ArrowUp),
        KCode::ArrowRight => Some(ICode::ArrowRight),
        KCode::ArrowDown => Some(ICode::ArrowDown),

        KCode::Backspace => Some(ICode::Backspace),
        KCode::Enter => Some(ICode::Enter),
        KCode::Space => Some(ICode::Space),

        KCode::NumLock => Some(ICode::NumLock),
        KCode::Numpad0 => Some(ICode::Numpad0),
        KCode::Numpad1 => Some(ICode::Numpad1),
        KCode::Numpad2 => Some(ICode::Numpad2),
        KCode::Numpad3 => Some(ICode::Numpad3),
        KCode::Numpad4 => Some(ICode::Numpad4),
        KCode::Numpad5 => Some(ICode::Numpad5),
        KCode::Numpad6 => Some(ICode::Numpad6),
        KCode::Numpad7 => Some(ICode::Numpad7),
        KCode::Numpad8 => Some(ICode::Numpad8),
        KCode::Numpad9 => Some(ICode::Numpad9),
        KCode::NumpadAdd => Some(ICode::NumpadAdd),
        KCode::NumpadDivide => Some(ICode::NumpadDivide),
        KCode::NumpadDecimal => Some(ICode::NumpadDecimal),
        KCode::NumpadComma => Some(ICode::NumpadComma),
        KCode::NumpadEnter => Some(ICode::NumpadEnter),
        KCode::NumpadEqual => Some(ICode::NumpadEqual),
        KCode::NumpadMultiply => Some(ICode::NumpadMultiply),
        KCode::NumpadSubtract => Some(ICode::NumpadSubtract),

        KCode::Convert => Some(ICode::Convert),
        KCode::KanaMode => Some(ICode::KanaMode),

        KCode::NonConvert => Some(ICode::NonConvert),
        KCode::IntlYen => Some(ICode::IntlYen),

        KCode::AltLeft => Some(ICode::AltLeft),
        KCode::AltRight => Some(ICode::AltRight),
        KCode::BracketLeft => Some(ICode::BracketLeft),
        KCode::BracketRight => Some(ICode::BracketRight),
        KCode::ControlLeft => Some(ICode::ControlLeft),
        KCode::ControlRight => Some(ICode::ControlRight),
        KCode::ShiftLeft => Some(ICode::ShiftLeft),
        KCode::ShiftRight => Some(ICode::ShiftRight),
        KCode::MetaLeft => Some(ICode::Meta),
        KCode::MetaRight => Some(ICode::Meta),

        KCode::Minus => Some(ICode::Minus),
        KCode::Period => Some(ICode::Period),

        KCode::Equal => Some(ICode::Equal),
        KCode::Quote => Some(ICode::Quote),
        KCode::Comma => Some(ICode::Comma),

        KCode::Semicolon => Some(ICode::Semicolon),
        KCode::Backslash => Some(ICode::Backslash),
        KCode::Slash => Some(ICode::Slash),
        KCode::Tab => Some(ICode::Tab),

        KCode::Copy => Some(ICode::Copy),
        KCode::Paste => Some(ICode::Paste),
        KCode::Cut => Some(ICode::Cut),

        KCode::MediaSelect => Some(ICode::MediaSelect),
        KCode::MediaStop => Some(ICode::MediaStop),
        KCode::MediaPlayPause => Some(ICode::MediaPlayPause),
        KCode::AudioVolumeMute => Some(ICode::AudioVolumeMute),
        KCode::AudioVolumeDown => Some(ICode::AudioVolumeDown),
        KCode::AudioVolumeUp => Some(ICode::AudioVolumeUp),
        KCode::MediaTrackNext => Some(ICode::MediaTrackNext),
        KCode::MediaTrackPrevious => Some(ICode::MediaTrackPrevious),

        _ => None,
    }
}

pub fn convert_mouse_interaction(
    interaction: crate::iced::runtime::core::mouse::Interaction,
) -> crate::MouseCursor {
    use crate::MouseCursor as BCursor;
    use crate::iced::runtime::core::mouse::Interaction as ICursor;

    match interaction {
        ICursor::None => BCursor::Default,
        ICursor::Idle => BCursor::Default,
        ICursor::Pointer => BCursor::Hand,
        ICursor::Grab => BCursor::HandGrabbing,
        ICursor::Text => BCursor::Text,
        ICursor::Crosshair => BCursor::Crosshair,
        ICursor::Progress => BCursor::Working,
        ICursor::Grabbing => BCursor::HandGrabbing,
        ICursor::ResizingHorizontally => BCursor::ColResize,
        ICursor::ResizingVertically => BCursor::RowResize,
        ICursor::ResizingDiagonallyUp => BCursor::NeswResize,
        ICursor::ResizingDiagonallyDown => BCursor::NwseResize,
        ICursor::NotAllowed => BCursor::NotAllowed,
        ICursor::ZoomIn => BCursor::ZoomIn,
        ICursor::ZoomOut => BCursor::ZoomOut,
        ICursor::Cell => BCursor::Cell,
        ICursor::Move => BCursor::Move,
        ICursor::Copy => BCursor::Copy,
        ICursor::Help => BCursor::Help,
        ICursor::Wait => BCursor::Working,
        ICursor::Hidden => BCursor::Hidden,
        ICursor::ContextMenu => BCursor::Default,
        ICursor::Alias => BCursor::Alias,
        ICursor::NoDrop => BCursor::NotAllowed,
        ICursor::ResizingColumn => BCursor::ColResize,
        ICursor::ResizingRow => BCursor::RowResize,
        ICursor::AllScroll => BCursor::AllScroll,
    }
}

#[derive(Clone)]
pub struct WindowWrapper {
    window: raw_window_handle::RawWindowHandle,
    display: raw_window_handle::RawDisplayHandle,
}

pub fn convert_window(window: &crate::Window<'_>) -> WindowWrapper {
    WindowWrapper {
        window: window.window_handle().unwrap().as_raw(),
        display: window.display_handle().unwrap().as_raw(),
    }
}

impl raw_window_handle::HasWindowHandle for WindowWrapper {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'static>, raw_window_handle::HandleError> {
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(self.window) })
    }
}

impl raw_window_handle::HasDisplayHandle for WindowWrapper {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'static>, raw_window_handle::HandleError> {
        Ok(unsafe { raw_window_handle::DisplayHandle::borrow_raw(self.display) })
    }
}

unsafe impl Send for WindowWrapper {}
unsafe impl Sync for WindowWrapper {}
