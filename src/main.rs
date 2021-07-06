extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use glutin_window::GlutinWindow as Window;
use graphics::{clear, ellipse, polygon, rectangle, Context, Transformed};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use std::f64::consts::PI;

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    entities: Vec<Entity>,
    background_color: [f32; 4],
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        let background_color: [f32; 4] = self.background_color.clone();

        self.gl.draw(args.viewport(), |_c, gl| {
            // Clear the screen.
            clear(background_color, gl);
        });

        for entity in self.entities.iter() {
            self.gl.draw(args.viewport(), |c, gl| {
                // Clear the screen.
                entity.renderer.draw(gl, c, entity.physics.clone());
            });
        }
    }

    fn update(&mut self, _args: &UpdateArgs) {
        for entity in self.entities.iter_mut() {
            match &entity.behavior {
                Some(_behavior) => {
                    let ai = entity.behavior.take().unwrap();

                    ai.apply_behavior(entity);
                    entity.behavior = Some(Box::new(Walker {}));
                }
                None => {}
            }
        }
    }
}

struct Entity {
    physics: Physics,
    renderer: Box<dyn Drawable>,
    behavior: Option<Box<dyn AI>>,
    _id: u32,
}

#[derive(Clone, Debug, Copy, PartialEq)]
struct Physics {
    x: f64,
    y: f64,
    size: f64,
    rotation: f64,
}

trait Drawable {
    fn draw(&self, gl: &mut GlGraphics, c: Context, physics: Physics);
}

struct Square {
    color: [f32; 4],
}

impl Drawable for Square {
    fn draw(&self, gl: &mut GlGraphics, c: Context, physics: Physics) {
        let square = rectangle::square(0.0, 0.0, physics.size);
        let x = physics.x - physics.size / 2.0;
        let y = physics.y - physics.size / 2.0;
        let transform = c.transform.trans(x, y).rot_rad(physics.rotation);

        rectangle(self.color.clone(), square, transform, gl);
    }
}

struct Arrow {
    color: [f32; 4],
}

impl Drawable for Arrow {
    fn draw(&self, gl: &mut GlGraphics, c: Context, physics: Physics) {
        let x = physics.x;
        let y = physics.y - physics.size / 2.0;
        let transform = c.transform.trans(x, y).rot_rad(physics.rotation);

        polygon(
            self.color.clone(),
            &[
                [0.0, -physics.size / 2.0],
                [-physics.size / 3.0, physics.size / 2.0],
                [physics.size / 3.0, physics.size / 2.0],
            ],
            transform,
            gl,
        );
    }
}

struct Circle {
    color: [f32; 4],
}

impl Drawable for Circle {
    fn draw(&self, gl: &mut GlGraphics, c: Context, physics: Physics) {
        let square = rectangle::square(0.0, 0.0, physics.size);
        let x = physics.x - physics.size / 2.0;
        let y = physics.y - physics.size / 2.0;
        let transform = c.transform.trans(x, y);

        ellipse(self.color.clone(), square, transform, gl);
    }
}

trait AI {
    fn apply_behavior(&self, entity: &mut Entity);
}

struct Walker {}

impl AI for Walker {
    fn apply_behavior(&self, entity: &mut Entity) {
        use rand::Rng;
        use rand_distr::{Distribution, Normal};
        let normal = Normal::new(2.0, 1.0).unwrap();

        let mut rng = rand::thread_rng();
        let random_x_direction: i16 = rng.gen_range(-1..2);
        let random_y_direction: i16 = rng.gen_range(-1..2);
        let speed: f64 = normal.sample(&mut rand::thread_rng());
        entity.physics.x += random_x_direction as f64 * speed;
        entity.physics.y += random_y_direction as f64 * speed;

        match random_x_direction {
            -1 => match random_y_direction {
                -1 => {
                    entity.physics.rotation = -PI / 4.0;
                }
                1 => {
                    entity.physics.rotation = -3.0 * PI / 4.0;
                }
                _ => {
                    entity.physics.rotation = -PI / 2.0;
                }
            },
            1 => match random_y_direction {
                -1 => {
                    entity.physics.rotation = PI / 4.0;
                }
                1 => {
                    entity.physics.rotation = 3.0 * PI / 4.0;
                }
                _ => {
                    entity.physics.rotation = PI / 2.0;
                }
            },
            _ => match random_y_direction {
                -1 => {
                    entity.physics.rotation = 0.0;
                }
                1 => {
                    entity.physics.rotation = PI;
                }
                _ => {}
            },
        }
    }
}

fn color_generator(x: f32, y: f32, width: u32, height: u32) -> [f32; 4] {
    let red = x / width as f32;
    let green = y / height as f32;
    // let blue = (x + y) / (height as f32 + width as f32);

    [red, green, 0.0, 1.0]
}

fn gaussian_dots_generator(size: usize, width: u32, height: u32) -> Vec<Entity> {
    use rand_distr::{Distribution, Normal};
    let mut entities: Vec<Entity> = Vec::with_capacity(size);
    let y_normal = Normal::new(height as f64 / 2.0, height as f64 / 6.0).unwrap();
    let x_normal = Normal::new(width as f64 / 2.0, width as f64 / 6.0).unwrap();

    for id in 0..300 {
        let x = x_normal.sample(&mut rand::thread_rng());
        let y = y_normal.sample(&mut rand::thread_rng());

        entities.push(Entity {
            physics: Physics {
                x,
                y,
                size: 10.0,
                rotation: 0.0,
            },
            renderer: Box::new(Circle {
                color: color_generator(x as f32, y as f32, width, height),
            }),
            behavior: Some(Box::new(Walker {})),
            _id: id,
        });
    }

    entities
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;
    let height = 500;
    let width = 800;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("Window", [width, height])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        entities: gaussian_dots_generator(300, width, height),
        background_color: [0.0, 1.0, 0.0, 1.0],
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}
