#![deny(warnings)]

mod painter;

pub use painter::Painter;

use {
    clipboard::{ClipboardContext, ClipboardProvider},
    emigui::*,
    glium::glutin::{self, VirtualKeyCode},
};

pub fn init_clipboard() -> Option<ClipboardContext> {
    match ClipboardContext::new() {
        Ok(clipboard) => Some(clipboard),
        Err(err) => {
            eprintln!("Failed to initialize clipboard: {}", err);
            None
        }
    }
}

pub fn input_event(
    event: glutin::Event,
    clipboard: Option<&mut ClipboardContext>,
    raw_input: &mut RawInput,
    running: &mut bool,
) {
    use glutin::WindowEvent::*;
    match event {
        glutin::Event::WindowEvent { event, .. } => match event {
            CloseRequested | Destroyed => *running = false,

            DroppedFile(path) => raw_input.dropped_files.push(path),
            HoveredFile(path) => raw_input.hovered_files.push(path),

            Resized(glutin::dpi::LogicalSize { width, height }) => {
                raw_input.screen_size = vec2(width as f32, height as f32);
            }
            MouseInput { state, .. } => {
                raw_input.mouse_down = state == glutin::ElementState::Pressed;
            }
            CursorMoved { position, .. } => {
                raw_input.mouse_pos = Some(pos2(position.x as f32, position.y as f32));
            }
            CursorLeft { .. } => {
                raw_input.mouse_pos = None;
            }
            ReceivedCharacter(ch) => {
                raw_input.events.push(Event::Text(ch.to_string()));
            }
            KeyboardInput { input, .. } => {
                if let Some(virtual_keycode) = input.virtual_keycode {
                    // TODO: If mac
                    if input.modifiers.logo && virtual_keycode == VirtualKeyCode::Q {
                        *running = false;
                    }

                    match virtual_keycode {
                        VirtualKeyCode::Paste => {
                            if let Some(clipboard) = clipboard {
                                match clipboard.get_contents() {
                                    Ok(contents) => {
                                        raw_input.events.push(Event::Text(contents));
                                    }
                                    Err(err) => {
                                        eprintln!("Paste error: {}", err);
                                    }
                                }
                            }
                        }
                        VirtualKeyCode::Copy => raw_input.events.push(Event::Copy),
                        VirtualKeyCode::Cut => raw_input.events.push(Event::Cut),
                        _ => {
                            if let Some(key) = translate_virtual_key_code(virtual_keycode) {
                                raw_input.events.push(Event::Key {
                                    key,
                                    pressed: input.state == glutin::ElementState::Pressed,
                                });
                            }
                        }
                    }
                }
            }
            MouseWheel { delta, .. } => {
                match delta {
                    glutin::MouseScrollDelta::LineDelta(x, y) => {
                        raw_input.scroll_delta = vec2(x, y) * 24.0;
                    }
                    glutin::MouseScrollDelta::PixelDelta(delta) => {
                        // Actually point delta
                        raw_input.scroll_delta = vec2(delta.x as f32, delta.y as f32);
                    }
                }
            }
            // TODO: HiDpiFactorChanged
            _ => {
                // dbg!(event);
            }
        },
        _ => (),
    }
}

pub fn translate_virtual_key_code(key: glutin::VirtualKeyCode) -> Option<emigui::Key> {
    use VirtualKeyCode::*;

    Some(match key {
        Escape => Key::Escape,
        Insert => Key::Insert,
        Home => Key::Home,
        Delete => Key::Delete,
        End => Key::End,
        PageDown => Key::PageDown,
        PageUp => Key::PageUp,
        Left => Key::Left,
        Up => Key::Up,
        Right => Key::Right,
        Down => Key::Down,
        Back => Key::Backspace,
        Return => Key::Return,
        // Space => Key::Space,
        Tab => Key::Tab,

        LAlt | RAlt => Key::Alt,
        LShift | RShift => Key::Shift,
        LControl | RControl => Key::Control,
        LWin | RWin => Key::Logo,

        _ => {
            return None;
        }
    })
}

pub fn translate_cursor(cursor_icon: emigui::CursorIcon) -> glutin::MouseCursor {
    match cursor_icon {
        CursorIcon::Default => glutin::MouseCursor::Default,
        CursorIcon::PointingHand => glutin::MouseCursor::Hand,
        CursorIcon::ResizeNwSe => glutin::MouseCursor::NwseResize,
        CursorIcon::Text => glutin::MouseCursor::Text,
    }
}

pub fn handle_output(
    output: emigui::Output,
    display: &glium::backend::glutin::Display,
    clipboard: Option<&mut ClipboardContext>,
) {
    if let Some(url) = output.open_url {
        if let Err(err) = webbrowser::open(&url) {
            eprintln!("Failed to open url: {}", err); // TODO show error in imgui
        }
    }

    if !output.copied_text.is_empty() {
        if let Some(clipboard) = clipboard {
            if let Err(err) = clipboard.set_contents(output.copied_text) {
                eprintln!("Copy/Cut error: {}", err);
            }
        }
    }

    display
        .gl_window()
        .set_cursor(translate_cursor(output.cursor_icon));
}
