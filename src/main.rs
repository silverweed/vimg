#![windows_subsystem = "windows"]

use sfml::graphics::{
    Color, Rect, RenderTarget, RenderWindow, Sprite, Texture, Transformable, View,
};
use sfml::system::{Vector2f, Vector2i};
use sfml::window::{ContextSettings, Event, Key, Style, VideoMode};
use sfml::window::mouse::Button;

const TITLE: &str = "vimg";

#[derive(Copy, Clone, Debug)]
enum WinStyle {
    Default,
    Fullscreen,
}

impl WinStyle {
    pub fn get_style(self) -> Style {
        match self {
            Self::Default => Style::default(),
            Self::Fullscreen => Style::NONE,
        }
    }
}

fn main() {
    let mut args = std::env::args();
    args.next();
    let img_path = args.next().unwrap();

    let desktop_mode = VideoMode::desktop_mode();
    let mut cur_style = WinStyle::Default;
    let ctx_settings = ContextSettings::default();

    let target_size = (desktop_mode.width, desktop_mode.height);
    let mut win = RenderWindow::new(desktop_mode, TITLE, cur_style.get_style(), &ctx_settings);
    // #TODO: maximize the window
    // let handle = win.system_handle();
    // unsafe { ShowWindow(handle, SW_MAXIMIZE); }
    win.set_position(Vector2i::default());
    win.set_vertical_sync_enabled(true);

    let mut scheduled_win_change: Option<RenderWindow> = None;

    let mut tex = Texture::from_file(&img_path).unwrap();
    tex.set_smooth(true);
    let mut sprite = Sprite::with_texture(&*tex);
    // Center and maximize image
    center_and_maximize(&win, &tex, &mut sprite);

    let win_real_size = win.size();
    let mut view = View::from_rect(&Rect::new(0.0, 0.0, win_real_size.x as f32, win_real_size.y as f32));
    win.set_view(&view);

    let mut dragging = false;
    let mut prev_mouse_pos = Vector2i::default();

    let mut dirty = true;

    loop {
        if let Some(event) = win.wait_event() {
            match event {
                Event::Closed | Event::KeyPressed { code: Key::Q, .. } => return,
                Event::KeyPressed { code, .. } => match code {
                    Key::F => {
                        cur_style = match cur_style {
                            WinStyle::Default => WinStyle::Fullscreen,
                            WinStyle::Fullscreen => WinStyle::Default,
                        };
                        scheduled_win_change = Some(RenderWindow::new(
                            desktop_mode,
                            TITLE,
                            cur_style.get_style(),
                            &ctx_settings,
                        ));
                    }
                    Key::Num0 | Key::Numpad0 => {
                        sprite.set_scale(Vector2f::new(1.0, 1.0));
                        dirty = true;
                    }
                    Key::R => {
                        center_and_maximize(&win, &tex, &mut sprite);
                        dirty = true;
                    }
                    _ => {}
                },
                Event::Resized { width, height } => {
                    let viewport = resize_keep_ratio(width, height, target_size);
                    let view_rect = Rect::new(0.0, 0.0, target_size.0 as f32, target_size.1 as f32);
                    let mut view = View::from_rect(&view_rect);
                    view.set_viewport(&viewport);
                    win.set_view(&view);
                    dirty = true;
                }
                Event::MouseWheelScrolled { delta, .. } => {
                    let factor = if delta < 0.0 { 1.1 } else { 0.9 };
                    view.zoom(factor);
                    win.set_view(&view);
                    dirty = true;
                }
                Event::MouseButtonPressed {
                    button: Button::Left,
                    ..
                } => {
                    dragging = true;
                }
                Event::MouseButtonReleased {
                    button: Button::Left,
                    ..
                } => {
                    dragging = false;
                }
                Event::MouseMoved {
                    x, y
                } => {
                    let cur = Vector2i::new(x, y);
                    if dragging {
                        let delta =  cur - prev_mouse_pos;
                        view.move_(-Vector2f::new(delta.x as f32, delta.y as f32));
                        win.set_view(&view);
                        dirty = true;
                    }
                    prev_mouse_pos = cur;
                }
                Event::LostFocus => {
                    dragging = false;
                }
                _ => {}
            }

            if let Some(win_change) = scheduled_win_change.take() {
                win = win_change;
                win.set_position(Vector2i::default());
                win.set_vertical_sync_enabled(true);
                dirty = true;
            }

            if dirty {
                win.clear(Color::BLACK);
                win.draw(&sprite);
                win.display();
                dirty = false;
            }
        }
    }
}

fn resize_keep_ratio(
    new_width: u32,
    new_height: u32,
    (target_width, target_height): (u32, u32),
) -> Rect<f32> {
    use std::cmp::Ordering;

    if new_width == 0 || new_height == 0 {
        return Rect::default();
    }

    assert!(target_width != 0 && target_height != 0);

    let screen_width = new_width as f32 / target_width as f32;
    let screen_height = new_height as f32 / target_height as f32;

    let mut viewport = Rect::new(0.0, 0.0, 1.0, 1.0);
    match screen_width.partial_cmp(&screen_height) {
        Some(Ordering::Greater) => {
            viewport.width = screen_height / screen_width;
            viewport.left = 0.5 * (1.0 - viewport.width);
        }
        Some(Ordering::Less) => {
            viewport.height = screen_width / screen_height;
            viewport.top = 0.5 * (1.0 - viewport.height);
        }
        _ => {}
    }

    viewport
}

fn center_and_maximize(win: &RenderWindow, tex: &Texture, sprite: &mut Sprite) {
    let win_real_size = win.size();
    let tex_size = tex.size();
    let y_ratio = win_real_size.y as f32 / tex_size.y as f32;
    let x_ratio = win_real_size.x as f32 / tex_size.x as f32;
    if y_ratio < x_ratio {
        let sw = (y_ratio * tex_size.x as f32) as u32;
        let offx = (win_real_size.x - sw) / 2;
        sprite.set_position(Vector2f::new(offx as f32, 0.0));
        sprite.set_scale(Vector2f::new(y_ratio, y_ratio));
    } else {
        let sh = (x_ratio * tex_size.y as f32) as u32;
        let offy = (win_real_size.y - sh) / 2;
        sprite.set_position(Vector2f::new(0.0, offy as f32));
        sprite.set_scale(Vector2f::new(x_ratio, x_ratio));
    }
}
