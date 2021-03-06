extern crate native;
extern crate kiss3d;
extern crate nalgebra;

use std::rc::Rc;
use std::cell::RefCell;
use nalgebra::na;
use nalgebra::na::Vec3;
use kiss3d::window::Window;
use kiss3d::resource::{Mesh, MeshManager};
use kiss3d::light;

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    let mut window = Window::new("Kiss3d: custom_mesh_shared");

    let a = Vec3::new(-1.0, -1.0, 0.0);
    let b = Vec3::new(1.0, -1.0, 0.0);
    let c = Vec3::new(0.0, 1.0, 0.0);

    let vertices = vec!(a, b, c);
    let indices  = vec!(Vec3::new(0u32, 1, 2));

    let mesh = Rc::new(RefCell::new(Mesh::new(vertices, indices, None, None, false)));

    // XXX: it would be better to do: MeshManager::add(Rc....) directly.
    MeshManager::get_global_manager(|mm| mm.add(mesh.clone(), "custom_mesh"));

    let mut c1 = window.add_geom_with_name("custom_mesh", na::one()).unwrap();
    let mut c2 = window.add_geom_with_name("custom_mesh", na::one()).unwrap();

    c1.set_color(1.0, 0.0, 0.0);
    c2.set_color(0.0, 1.0, 0.0);

    c1.enable_backface_culling(false);
    c2.enable_backface_culling(false);

    window.set_light(light::StickToCamera);

    for _ in window.iter() {
        c1.prepend_to_local_rotation(&Vec3::new(0.0f32, 0.014, 0.0));
        c2.prepend_to_local_rotation(&Vec3::new(0.0f32, -0.014, 0.0))
    }
}
