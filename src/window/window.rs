//! The kiss3d window.
/*
 * FIXME: this file is too big. Some heavy refactoring need to be done here.
 */

use glfw;
use glfw::Context;
use std::io::timer::Timer;
use std::cell::RefCell;
use std::rc::Rc;
use libc;
use time;
use gl;
use gl::types::*;
use nalgebra::na::{Vec2, Vec3};
use nalgebra::na;
use ncollide::procedural::TriMesh;
use camera::Camera;
use scene::SceneNode;
use line_renderer::LineRenderer;
use point_renderer::PointRenderer;
use post_processing::PostProcessingEffect;
use resource::{FramebufferManager, RenderTarget, Texture, TextureManager, Mesh, Material};
use light::{Light, Absolute, StickToCamera};
use text::{TextRenderer, Font};
use window::RenderFrames;
use camera::ArcBall;


static DEFAULT_WIDTH:  u32 = 800u32;
static DEFAULT_HEIGHT: u32 = 600u32;

/// Structure representing a window and a 3D scene.
///
/// This is the main interface with the 3d engine.
pub struct Window<'a> {
    events:                     Rc<Receiver<(f64, glfw::WindowEvent)>>,
    glfw:                       glfw::Glfw,
    window:                     glfw::Window,
    max_ms_per_frame:           Option<u64>,
    scene:                      SceneNode,
    light_mode:                 Light, // FIXME: move that to the scene graph
    background:                 Vec3<GLfloat>,
    line_renderer:              LineRenderer,
    point_renderer:             PointRenderer,
    text_renderer:              TextRenderer,
    framebuffer_manager:        FramebufferManager,
    post_process_render_target: RenderTarget,
    timer:                      Timer,
    curr_time:                  u64
}

impl<'a> Window<'a> {
    // FIXME: this doc is sooo unhelpful.
    /// Gets an iterator to the rendering frames.
    ///
    /// Uses this to start your render loop. The `RenderFrames` iterator iters though the frames
    /// that are drawn on the window.
    #[inline]
    pub fn iter<'b>(&'b mut self) -> RenderFrames<'b, 'a, ArcBall> {
        self.iter_with_camera(ArcBall::new(-Vec3::z(), na::zero()))
    }

    /// Gets an iterator to the rendering frames with the camera `camera`.
    ///
    /// Uses this to start your render loop. The `RenderFrames` iterator iters though the frames
    /// that are drawn on the window.
    #[inline]
    pub fn iter_with_camera<'b, C: 'static + Camera>(&'b mut self, camera: C) -> RenderFrames<'b, 'a, C> {
        let events = self.events.clone();

        RenderFrames::new_with_camera(events, self, camera)
    }

    /// Access the glfw context.
    #[inline]
    pub fn context<'r>(&'r self) -> &'r glfw::Glfw {
        &'r self.glfw
    }

    /// Access the glfw window.
    #[inline]
    pub fn glfw_window<'r>(&'r self) -> &'r glfw::Window {
        &'r self.window
    }

    /// The window width.
    #[inline]
    pub fn width(&self) -> f32 {
        let (w, _) = self.window.get_size();

        w as f32
    }

    /// The window height.
    #[inline]
    pub fn height(&self) -> f32 {
        let (_, h) = self.window.get_size();

        h as f32
    }

    /// The size of the window.
    #[inline]
    pub fn size(&self) -> Vec2<f32> {
        let (w, h) = self.window.get_size();

        Vec2::new(w as f32, h as f32)
    }

    /// Sets the maximum number of frames per second. Cannot be 0. `None` means there is no limit.
    #[inline]
    pub fn set_framerate_limit(&mut self, fps: Option<u64>) {
        self.max_ms_per_frame = fps.map(|f| { assert!(f != 0); 1000 / f })
    }

    /// Closes the window.
    #[inline]
    pub fn close(&mut self) {
        self.window.set_should_close(true)
    }

    /// Hides the window, without closing it. Use `show` to make it visible again.
    #[inline]
    pub fn hide(&mut self) {
        self.window.hide()
    }

    /// Makes the window visible. Use `hide` to hide it.
    #[inline]
    pub fn show(&mut self) {
        self.window.show()
    }

    /// Sets the background color.
    #[inline]
    pub fn set_background_color(&mut self, r: f32, g: GLfloat, b: f32) {
        self.background.x = r;
        self.background.y = g;
        self.background.z = b;
    }

    // XXX: remove this (moved to the render_frame).
    /// Adds a line to be drawn during the next frame.
    #[inline]
    pub fn draw_line(&mut self, a: &Vec3<f32>, b: &Vec3<f32>, color: &Vec3<f32>) {
        self.line_renderer.draw_line(a.clone(), b.clone(), color.clone());
    }

    // XXX: remove this (moved to the render_frame).
    /// Adds a point to be drawn during the next frame.
    #[inline]
    pub fn draw_point(&mut self, pt: &Vec3<f32>, color: &Vec3<f32>) {
        self.point_renderer.draw_point(pt.clone(), color.clone());
    }

    // XXX: remove this (moved to the render_frame).
    /// Adds a string to be drawn during the next frame.
    #[inline]
    pub fn draw_text(&mut self, text: &str, pos: &Vec2<f32>, font: &Rc<Font>, color: &Vec3<f32>) {
        self.text_renderer.draw_text(text, pos, font, color);
    }

    /// Removes an object from the scene.
    pub fn remove(&mut self, sn: &mut SceneNode) {
        sn.unlink()
    }

    /// Adds a group to the scene.
    ///
    /// A group is a node not containing any object.
    pub fn add_group(&mut self) -> SceneNode {
        self.scene.add_group()
    }

    /// Adds an obj model to the scene.
    ///
    /// # Arguments
    /// * `path`  - relative path to the obj file.
    /// * `scale` - scale to apply to the model.
    pub fn add_obj(&mut self, path: &Path, mtl_dir: &Path, scale: Vec3<f32>) -> SceneNode {
        self.scene.add_obj(path, mtl_dir, scale)
    }

    /// Adds an unnamed mesh to the scene.
    pub fn add_mesh(&mut self, mesh: Rc<RefCell<Mesh>>, scale: Vec3<f32>) -> SceneNode {
        self.scene.add_mesh(mesh, scale)
    }

    /// Creates and adds a new object using the geometry generated by a given procedural generator.
    /// Creates and adds a new object using a mesh descriptor.
    pub fn add_trimesh(&mut self, descr: TriMesh<f32, Vec3<f32>>, scale: Vec3<f32>) -> SceneNode {
        self.scene.add_trimesh(descr, scale)
    }

    /// Creates and adds a new object using the geometry registered as `geometry_name`.
    pub fn add_geom_with_name(&mut self, geometry_name: &str, scale: Vec3<f32>) -> Option<SceneNode> {
        self.scene.add_geom_with_name(geometry_name, scale)
    }

    /// Adds a cube to the scene. The cube is initially axis-aligned and centered at (0, 0, 0).
    ///
    /// # Arguments
    /// * `wx` - the cube extent along the z axis
    /// * `wy` - the cube extent along the y axis
    /// * `wz` - the cube extent along the z axis
    pub fn add_cube(&mut self, wx: GLfloat, wy: GLfloat, wz: GLfloat) -> SceneNode {
        self.scene.add_cube(wx, wy, wz)
    }

    /// Adds a sphere to the scene. The sphere is initially centered at (0, 0, 0).
    ///
    /// # Arguments
    /// * `r` - the sphere radius
    pub fn add_sphere(&mut self, r: GLfloat) -> SceneNode {
        self.scene.add_sphere(r)
    }

    /// Adds a cone to the scene. The cone is initially centered at (0, 0, 0) and points toward the
    /// positive `y` axis.
    ///
    /// # Arguments
    /// * `h` - the cone height
    /// * `r` - the cone base radius
    pub fn add_cone(&mut self, r: GLfloat, h: GLfloat) -> SceneNode {
        self.scene.add_cone(r, h)
    }

    /// Adds a cylinder to the scene. The cylinder is initially centered at (0, 0, 0) and has its
    /// principal axis aligned with the `y` axis.
    ///
    /// # Arguments
    /// * `h` - the cylinder height
    /// * `r` - the cylinder base radius
    pub fn add_cylinder(&mut self, r: GLfloat, h: GLfloat) -> SceneNode {
        self.scene.add_cylinder(r, h)
    }

    /// Adds a capsule to the scene. The capsule is initially centered at (0, 0, 0) and has its
    /// principal axis aligned with the `y` axis.
    ///
    /// # Arguments
    /// * `h` - the capsule height
    /// * `r` - the capsule caps radius
    pub fn add_capsule(&mut self, r: GLfloat, h: GLfloat) -> SceneNode {
        self.scene.add_capsule(r, h)
    }

    /// Adds a double-sided quad to the scene. The quad is initially centered at (0, 0, 0). The
    /// quad itself is composed of a user-defined number of triangles regularly spaced on a grid.
    /// This is the main way to draw height maps.
    ///
    /// # Arguments
    /// * `w` - the quad width.
    /// * `h` - the quad height.
    /// * `wsubdivs` - number of horizontal subdivisions. This correspond to the number of squares
    /// which will be placed horizontally on each line. Must not be `0`.
    /// * `hsubdivs` - number of vertical subdivisions. This correspond to the number of squares
    /// which will be placed vertically on each line. Must not be `0`.
    /// update.
    pub fn add_quad(&mut self, w: f32, h: f32, usubdivs: uint, vsubdivs: uint) -> SceneNode {
        self.scene.add_quad(w, h, usubdivs, vsubdivs)
    }

    /// Adds a double-sided quad with the specified vertices.
    pub fn add_quad_with_vertices(&mut self,
                                  vertices: &[Vec3<f32>],
                                  nhpoints: uint,
                                  nvpoints: uint)
                                  -> SceneNode {
        self.scene.add_quad_with_vertices(vertices, nhpoints, nvpoints)
    }

    #[doc(hidden)]
    pub fn add_texture(&mut self, path: &Path, name: &str) -> Rc<Texture> {
        TextureManager::get_global_manager(|tm| tm.add(path, name))
    }

    /// Handles an event with the given camera.
    ///
    /// This will handle the `FramebufferSizeEvent` to update the viewport size, and the `Escape`
    /// key release event to close the window.
    ///
    /// Returns `true` if an event has been handled.
    pub fn handle_event(&mut self, event: &glfw::WindowEvent) -> bool {
        match *event {
            glfw::KeyEvent(glfw::KeyEscape, _, glfw::Release, _) => {
                self.close();
                true
            },
            glfw::FramebufferSizeEvent(w, h) => {
                self.update_viewport(w as f32, h as f32);
                true
            },
            _ => false
        }
    }

    /// Returns whether this window is closed or not.
    pub fn is_closed(&self) -> bool {
        false // FIXME
    }

    /// Sets the light mode. Only one light is supported.
    pub fn set_light(&mut self, pos: Light) {
        self.light_mode = pos;
    }

    /// Opens a window, hide it then calls a user-defined procedure.
    ///
    /// # Arguments
    /// * `title` - the window title
    pub fn new_hidden(title: &str) -> Window {
        Window::do_new(title, true, DEFAULT_WIDTH, DEFAULT_HEIGHT)
    }

    /// Opens a window then calls a user-defined procedure.
    ///
    /// # Arguments
    /// * `title` - the window title
    pub fn new(title: &str) -> Window {
        Window::do_new(title, false, DEFAULT_WIDTH, DEFAULT_HEIGHT)
    }

    /// Opens a window with a custom size then calls a user-defined procedure.
    ///
    /// # Arguments
    /// * `title` - the window title.
    /// * `width` - the window width.
    /// * `height` - the window height.
    pub fn new_with_size(title: &str, width: u32, height: u32) -> Window {
        Window::do_new(title, false, width, height)
    }

    // FIXME: make this pub?
    fn do_new(title: &str, hide: bool, width: u32, height: u32) -> Window {
        // FIXME: glfw::set_error_callback(~ErrorCallback);

        let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

        let (window, events) = glfw.create_window(width, height, title, glfw::Windowed).expect("Unable to open a glfw window.");

        window.make_current();

        verify!(gl::load_with(|name| glfw.get_proc_address(name)));
        init_gl();

        let mut usr_window = Window {
            max_ms_per_frame:      None,
            glfw:                  glfw,
            window:                window,
            events:                Rc::new(events),
            scene:                 SceneNode::new_empty(),
            light_mode:            Absolute(Vec3::new(0.0, 10.0, 0.0)),
            background:            Vec3::new(0.0, 0.0, 0.0),
            line_renderer:         LineRenderer::new(),
            point_renderer:        PointRenderer::new(),
            text_renderer:         TextRenderer::new(),
            post_process_render_target: FramebufferManager::new_render_target(width as uint, height as uint),
            framebuffer_manager:   FramebufferManager::new(),
            timer:                 Timer::new().unwrap(),
            curr_time:             time::precise_time_ns()
        };

        // setup callbacks
        usr_window.window.set_framebuffer_size_polling(true);
        usr_window.window.set_key_polling(true);
        usr_window.window.set_mouse_button_polling(true);
        usr_window.window.set_cursor_pos_polling(true);
        usr_window.window.set_scroll_polling(true);

        if hide {
            usr_window.window.hide()
        }

        // usr_window.framebuffer_size_callback(DEFAULT_WIDTH, DEFAULT_HEIGHT);
        let light = usr_window.light_mode.clone();
        usr_window.set_light(light);

        usr_window
    }

    /// Reference to the scene associated with this window.
    #[inline]
    pub fn scene<'a>(&'a self) -> &'a SceneNode {
        &'a self.scene
    }

    /// Mutable reference to the scene associated with this window.
    #[inline]
    pub fn scene_mut<'a>(&'a mut self) -> &'a mut SceneNode {
        &'a mut self.scene
    }

    // FIXME: give more options for the snap size and offset.
    /// Read the pixels currently displayed to the screen.
    ///
    /// # Arguments:
    /// * `out` - the output buffer. It is automatically resized.
    pub fn snap(&self, out: &mut Vec<u8>) {
        let (width, height) = self.window.get_size();

        let size = (width * height * 3) as uint;

        if out.len() < size {
            out.grow_set(size - 1, &0, 0);
        }
        else {
            out.truncate(size)
        }

        // FIXME: this is _not_ the fastest way of doing this.
        unsafe {
            gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
            gl::ReadPixels(0, 0,
                           width, height,
                           gl::RGB,
                           gl::UNSIGNED_BYTE,
                           out.get_mut(0) as *mut u8 as *mut libc::c_void);
        }
    }

    /// Draws the scene with the given camera and post-processing effect.
    pub fn draw(&mut self, camera: &mut Camera, post_processing: &mut Option<&mut PostProcessingEffect>) {
        camera.update(&self.window);

        match self.light_mode {
            StickToCamera => self.set_light(StickToCamera),
            _             => { }
        }

        if post_processing.is_some() {
            // if we need post-processing, render to our own frame buffer
            self.framebuffer_manager.select(&self.post_process_render_target);
        }
        else {
            self.framebuffer_manager.select(&FramebufferManager::screen());
        }

        for pass in range(0u, camera.num_passes()) {
            camera.start_pass(pass, &self.window);
            self.render_scene(camera, pass);
        }
        camera.render_complete(&self.window);

        let w = self.width();
        let h = self.height();
        let (znear, zfar) = camera.clip_planes();

        // FIXME: remove this completely?
        // swatch off the wireframe mode for post processing and text rendering.
        // if self.wireframe_mode {
        //     verify!(gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL));
        // }

        match *post_processing {
            Some(ref mut p) => {
                // switch back to the screen framebuffer …
                self.framebuffer_manager.select(&FramebufferManager::screen());
                // … and execute the post-process
                // FIXME: use the real time value instead of 0.016!
                p.update(0.016, w, h, znear, zfar);
                p.draw(&self.post_process_render_target);
            },
            None => { }
        }

        self.text_renderer.render(w, h);

        // We are done: swap buffers
        self.window.swap_buffers();

        // Limit the fps if needed.
        match self.max_ms_per_frame {
            None     => { },
            Some(ms) => {
                let elapsed = (time::precise_time_ns() - self.curr_time) / 1000000;
                if elapsed < ms {
                    self.timer.sleep(ms - elapsed);
                }
            }
        }

        self.curr_time = time::precise_time_ns();

        // self.transparent_objects.clear();
        // self.opaque_objects.clear();
    }

    fn render_scene(&mut self, camera: &mut Camera, pass: uint) {
        // Activate the default texture
        verify!(gl::ActiveTexture(gl::TEXTURE0));
        // Clear the screen to black
        verify!(gl::ClearColor(self.background.x, self.background.y, self.background.z, 1.0));
        verify!(gl::Clear(gl::COLOR_BUFFER_BIT));
        verify!(gl::Clear(gl::DEPTH_BUFFER_BIT));

        if self.line_renderer.needs_rendering() {
            self.line_renderer.render(pass, camera);
        }

        if self.point_renderer.needs_rendering() {
            self.point_renderer.render(pass, camera);
        }

        self.scene.data_mut().render(pass, camera, &self.light_mode);
    }


    fn update_viewport(&mut self, w: f32, h: f32) {
        // Update the viewport
        verify!(gl::Scissor(0, 0, w as i32, h as i32));
        FramebufferManager::screen().resize(w, h);
        self.post_process_render_target.resize(w, h);
    }
}

fn init_gl() {
    /*
     * Misc configurations
     */
    verify!(gl::FrontFace(gl::CCW));
    verify!(gl::Enable(gl::DEPTH_TEST));
    verify!(gl::Enable(gl::SCISSOR_TEST));
    verify!(gl::DepthFunc(gl::LEQUAL));
    verify!(gl::FrontFace(gl::CCW));
    verify!(gl::Enable(gl::CULL_FACE));
    verify!(gl::CullFace(gl::BACK));
}
