use std::error::Error;
use std::f32::consts::PI;
use std::path::PathBuf;
use std::{collections::HashSet, ops::Add};

use aot_plugin::{AlwaysOnTopPass, AlwaysOnTopPlugin};
use bevy::pbr::render_graph::PBR_PIPELINE_HANDLE;
use bevy::render::pipeline::RenderPipeline;
use bevy::{
    input::{
        mouse::{MouseButtonInput, MouseMotion},
        ElementState,
    },
    prelude::*,
    render::camera::Camera,
};
use cone::Cone;
use itertools::Itertools;
use structopt::StructOpt;

mod aot_plugin;
mod cone;

#[derive(Default, Debug)]
struct MouseButtonState {
    pressed: HashSet<MouseButton>,
    locked_position: Vec2,
}

#[derive(Default)]
struct Bulb {
    index: usize,
    inner: bool,
}

struct Frame {
    colors: Vec<Color>,
}

struct Sequence {
    frames: Vec<Frame>,
    time: f32,
    fps: f32,
}

struct BulbLocations(Vec<(f32, f32, f32)>);

#[derive(Debug, StructOpt)]
#[structopt(
    name = "xmas_tree_player",
    about = "Plays a christmas tree light sequence."
)]
struct Opt {
    #[structopt(parse(from_os_str))]
    sequence_path: PathBuf,
    #[structopt(parse(from_os_str), default_value = "coords/coords_2021.csv")]
    coords_path: PathBuf,
    #[structopt(long, default_value = "34.7")]
    fps: f32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    let mut led_coords_csv = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(opt.coords_path)?;
    let bulb_locations = BulbLocations(led_coords_csv.deserialize().collect::<Result<_, _>>()?);
    let mut sequence_csv = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(opt.sequence_path)?;
    let sequence = Sequence {
        frames: sequence_csv
            .records()
            .map(|record| {
                Ok(Frame {
                    colors: record?
                        .into_iter()
                        .skip(1)
                        .map(|f| f.parse::<f32>().expect("Expected number") / 255.0)
                        .tuples()
                        .map(|(r, g, b)| Color::rgb(r, g, b))
                        .collect(),
                })
            })
            .collect::<Result<_, csv::Error>>()?,
        time: 0.0,
        fps: opt.fps,
    };

    App::build()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(bulb_locations)
        .insert_resource(sequence)
        .init_resource::<MouseButtonState>()
        .add_plugins(DefaultPlugins)
        .add_plugin(AlwaysOnTopPlugin)
        .add_startup_system(setup.system())
        .add_system(mouse_button_input.system())
        .add_system(camera_control.system())
        .add_system(sequence_animation.system())
        .run();
    Ok(())
}

#[derive(Bundle)]
struct BulbBundle {
    bulb: Bulb,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    aot_pass: AlwaysOnTopPass,
    draw: Draw,
    visible: Visible,
    render_pipelines: RenderPipelines,
    transform: Transform,
    global_transform: GlobalTransform,
}

impl Default for BulbBundle {
    fn default() -> Self {
        Self {
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                PBR_PIPELINE_HANDLE.typed(),
            )]),
            bulb: Default::default(),
            mesh: Default::default(),
            visible: Default::default(),
            material: Default::default(),
            aot_pass: Default::default(),
            draw: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    bulb_locations: Res<BulbLocations>,
) {
    let bulb_mesh = meshes.add(Mesh::from(shape::Icosphere {
        radius: 0.01,
        subdivisions: 1,
    }));
    let glow_mesh = meshes.add(Mesh::from(shape::Icosphere {
        radius: 0.03,
        subdivisions: 1,
    }));
    for (index, &(x, y, z)) in bulb_locations.0.iter().enumerate() {
        commands.spawn_bundle(BulbBundle {
            bulb: Bulb { index, inner: true },
            mesh: bulb_mesh.clone(),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(1.0, 2.0, 3.0),
                unlit: true,
                ..Default::default()
            }),
            transform: Transform::from_xyz(x, z, y),
            ..Default::default()
        });
        commands.spawn_bundle(BulbBundle {
            bulb: Bulb {
                index,
                inner: false,
            },
            mesh: glow_mesh.clone(),
            material: materials.add(StandardMaterial {
                base_color: Color::rgba(0.0, 0.0, 0.6, 0.5),
                unlit: true,
                ..Default::default()
            }),
            visible: Visible {
                is_transparent: true,
                ..Default::default()
            },
            transform: Transform::from_xyz(x, z, y),
            ..Default::default()
        });
    }

    // cone
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(Cone {
            height: 2.5,
            radius: 0.9,
            ..Default::default()
        })),
        visible: Visible {
            is_transparent: true,
            ..Default::default()
        },
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.3, 0.5, 0.3).into(),
            roughness: 0.9,
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, 1.75, 0.0),
        ..Default::default()
    });
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(1.0, 0.5, 0.5).into()),
        ..Default::default()
    });
    // light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::Y * 1.5, Vec3::Y),
        ..Default::default()
    });
}

fn mouse_button_input(
    mut mouse_button_state: ResMut<MouseButtonState>,
    mut windows: ResMut<Windows>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
) {
    let window = windows.get_primary_mut().unwrap();
    let was_locked = !mouse_button_state.pressed.is_empty();
    for event in mouse_button_input_events.iter() {
        match event.state {
            ElementState::Pressed => {
                mouse_button_state.pressed.insert(event.button);
            }
            ElementState::Released => {
                mouse_button_state.pressed.remove(&event.button);
            }
        }
    }
    let is_locked = !mouse_button_state.pressed.is_empty();
    if is_locked {
        window.set_cursor_position(mouse_button_state.locked_position);
    } else {
        mouse_button_state.locked_position = window.cursor_position().unwrap_or_default();
    }
    if is_locked != was_locked {
        window.set_cursor_lock_mode(is_locked);
        window.set_cursor_visibility(!is_locked);
    }
}

fn camera_control(
    mouse_button_state: Res<MouseButtonState>,
    mut query: Query<&mut Transform, With<Camera>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
) {
    if mouse_button_state.pressed.contains(&MouseButton::Left) {
        let motion: Vec2 = mouse_motion_events
            .iter()
            .map(|e| e.delta)
            .fold(Vec2::ZERO, Add::add);
        let rotation = Quat::from_rotation_y((2.0 * PI / 1000.0) * motion.x);
        for mut transform in query.iter_mut() {
            *transform = Transform::from_rotation(rotation) * *transform;
        }
    }
}

fn sequence_animation(
    mut sequence: ResMut<Sequence>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    query: Query<(&Handle<StandardMaterial>, &Bulb)>,
) {
    sequence.time =
        (sequence.time + time.delta_seconds()) % (sequence.frames.len() as f32 / sequence.fps);
    let frame_index = (sequence.time * sequence.fps) as usize;
    let current_frame = &sequence.frames[frame_index];
    for (mat_handle, bulb) in query.iter() {
        let mat = materials.get_mut(mat_handle).unwrap();
        let mut color = current_frame.colors[bulb.index].as_hlsa_f32();
        if bulb.inner {
            color[2] = (color[2] + 0.5).min(1.0);
        } else {
            color[1] = (color[1] * 2.0).min(1.0);
            color[3] = color[2] * 0.5;
        };
        mat.base_color = Color::hsla(color[0], color[1], color[2], color[3]);
    }
}
