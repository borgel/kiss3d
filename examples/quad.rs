extern crate native;
extern crate rand;
extern crate kiss3d;
extern crate nalgebra;

use std::rand::random;
use kiss3d::window::Window;
use kiss3d::light;

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    let mut window = Window::new("Kiss3d: quad");

    let mut c = window.add_quad(5.0, 4.0, 500, 400);

    c.set_color(random(), random(), random());

    let mut time = 0.016f32;

    window.set_light(light::StickToCamera);

    for _ in window.iter() {
        c.modify_vertices(&mut |coords| {
            for v in coords.mut_iter() {
                v.z = time.sin() * (((v.x + time) * 4.0).cos() +
                                    time.sin() * ((v.y + time) * 4.0 + time).cos()) / 2.0
            }
        });
        c.recompute_normals();

        time = time + 0.016;
    }
}
