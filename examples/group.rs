extern crate native;
extern crate kiss3d;
extern crate nalgebra;

use nalgebra::na::Vec3;
use kiss3d::window::Window;
use kiss3d::light;

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    let mut window = Window::new("Kiss3d: cube");

    let mut g1 = window.add_group();
    let mut g2 = window.add_group();

    g1.append_translation(&Vec3::new(2.0f32, 0.0, 0.0));
    g2.append_translation(&Vec3::new(-2.0f32, 0.0, 0.0));

    g1.add_cube(1.0, 5.0, 1.0);
    g1.add_cube(5.0, 1.0, 1.0);

    g2.add_cube(1.0, 5.0, 1.0);
    g2.add_cube(1.0, 1.0, 5.0);

    g1.set_color(1.0, 0.0, 0.0);
    g2.set_color(0.0, 1.0, 0.0);

    window.set_light(light::StickToCamera);

    for _ in window.iter() {
        g1.prepend_to_local_rotation(&Vec3::new(0.0f32, 0.014, 0.0));
        g2.prepend_to_local_rotation(&Vec3::new(0.014f32, 0.0, 0.0))
    }
}
