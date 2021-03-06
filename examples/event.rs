extern crate native;
extern crate glfw;
extern crate kiss3d;
extern crate nalgebra;
extern crate debug;

use kiss3d::window::Window;

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    let mut window = Window::new("Kiss3d: events");

    for frame in window.iter() {
        for mut event in frame.events().iter() {
            match event.value {
                glfw::KeyEvent(code, _, glfw::Press, _) => {
                    println!("You pressed the key with code: {:?}", code);
                    println!("Do not try to press escape: the event is inhibited!");
                    event.inhibited = true // override the default keyboard handler
                },
                glfw::KeyEvent(code, _, glfw::Release, _) => {
                    println!("You released the key with code: {:?}", code);
                    println!("Do not try to press escape: the event is inhibited!");
                    event.inhibited = true // override the default keyboard handler
                },
                glfw::MouseButtonEvent(button, glfw::Press, mods) => {
                    println!("You pressed the mouse button with code: {:?}", button);
                    println!("You pressed the mouse button with modifiers: {:?}", mods);
                    // dont override the default mouse handler
                },
                glfw::MouseButtonEvent(button, glfw::Release, mods) => {
                    println!("You released the mouse button with code: {:?}", button);
                    println!("You released the mouse button with modifiers: {:?}", mods);
                    // dont override the default mouse handler
                },
                glfw::CursorPosEvent(x, y) => {
                    println!("Cursor pos: ({} , {})", x, y);
                    // dont override the default mouse handler
                },
                glfw::ScrollEvent(xshift, yshift) => {
                    println!("Cursor pos: ({} , {})", xshift, yshift);
                    // dont override the default mouse handler
                },
                _ => { }
            }
        }
    }
}
