use std::{error::Error, f32::consts::PI, io::stdout, path::PathBuf};

use rand::{
    prelude::{SliceRandom, StdRng},
    SeedableRng,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "xmas_tree_player",
    about = "Plays a christmas tree light sequence."
)]
struct Opt {
    effect: String,
    #[structopt(parse(from_os_str), default_value = "coords/coords_2021.csv")]
    coords_path: PathBuf,
    #[structopt(long, default_value = "1000")]
    len: usize,
}

type Coord = (f32, f32, f32);
type Color = (f32, f32, f32);

type EffectFn = fn(&[Coord], usize, usize) -> Vec<Color>;

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    let mut led_coords_csv = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(opt.coords_path)?;
    let coords: Vec<Coord> = led_coords_csv.deserialize().collect::<Result<_, _>>()?;

    let stdout = stdout();
    let mut sequence_csv = csv::Writer::from_writer(stdout.lock());

    sequence_csv.write_field("FRAME_ID")?;
    sequence_csv.write_record(
        (0..coords.len())
            .flat_map(|i| [format!("R_{}", i), format!("G_{}", i), format!("B_{}", i)]),
    )?;

    let effect_fn: EffectFn = match opt.effect.as_str() {
        "barber-pole" => barber_pole,
        "fill-up" => fill_up,
        "snake" => snake,
        "fall-down" => fall_down,
        "fall-down-rainbow" => fall_down_rainbow,
        "accelerate" => accelerate,
        "roll-around" => roll_around,
        "twinkle" => twinkle,
        other => {
            println!("Unknown effect: {}", other);
            return Ok(());
        }
    };
    for frame in 0..opt.len {
        sequence_csv.write_field(frame.to_string())?;
        sequence_csv.write_record(
            effect_fn(&coords, frame, opt.len)
                .into_iter()
                .flat_map(|color| [color.0, color.1, color.2])
                .map(|v| ((v * 255.0) as i32).to_string()),
        )?;
    }

    Ok(())
}

fn barber_pole(coords: &[Coord], frame: usize, total_frames: usize) -> Vec<Color> {
    let desired_speed = 0.05;
    let complete_cycles = ((total_frames as f32 * desired_speed) / (PI * 2.0)).floor();
    let actual_speed = (complete_cycles * PI * 2.0) / (total_frames as f32);
    let offset = frame as f32 * actual_speed;
    coords
        .iter()
        .map(|&(x, y, z)| {
            let angle = f32::atan2(x, y) + z * 5.0 + offset;
            if angle.sin() > 0.0 {
                (1.0, 0.0, 0.0)
            } else {
                (0.5, 0.5, 0.5)
            }
        })
        .collect()
}

fn saturated_color(hue: f32) -> (f32, f32, f32) {
    let r = hue.fract() * 6.0;
    if r < 1.0 {
        (1.0, r, 0.0)
    } else if r < 2.0 {
        (2.0 - r, 1.0, 0.0)
    } else if r < 3.0 {
        (0.0, 1.0, r - 2.0)
    } else if r < 4.0 {
        (0.0, 4.0 - r, 1.0)
    } else if r < 5.0 {
        (r - 4.0, 0.0, 1.0)
    } else {
        (1.0, 0.0, 6.0 - r)
    }
}

fn fill_up(coords: &[Coord], frame: usize, total_frames: usize) -> Vec<Color> {
    let desired_frames_per_fill = 60;
    let complete_fills = total_frames / desired_frames_per_fill;
    let frames_per_fill = total_frames / complete_fills;

    let color_seed0 = (frame * complete_fills) / total_frames;
    let color_seed1 = (color_seed0 + 1) % complete_fills;
    let color0 = saturated_color(color_seed0 as f32 * 0.45);
    let color1 = saturated_color(color_seed1 as f32 * 0.45);
    let max_height = coords.iter().map(|coord| coord.2).reduce(f32::max).unwrap();
    let base_frame = (color_seed0 * total_frames) / complete_fills;
    let height = (frame - base_frame) as f32 * max_height / (frames_per_fill as f32);
    coords
        .iter()
        .map(|&coord| if coord.2 > height { color0 } else { color1 })
        .collect()
}

fn snake(coords: &[Coord], frame: usize, _total_frames: usize) -> Vec<Color> {
    let snake_len = 20;
    let color = saturated_color(frame as f32 / 60.0);
    (0..coords.len())
        .map(|i| {
            let index = (i + frame) % coords.len();
            if index < snake_len {
                let f = 1.0 - index as f32 / (snake_len as f32);
                (color.0 * f, color.1 * f, color.2 * f)
            } else {
                (0.0, 0.0, 0.0)
            }
        })
        .collect()
}

fn fall_down(coords: &[Coord], frame: usize, total_frames: usize) -> Vec<Color> {
    let max_height = coords.iter().map(|coord| coord.2).reduce(f32::max).unwrap();
    let num_layers = 8;
    let layer_height = max_height / (num_layers as f32);
    let total_dist = (max_height + layer_height) * (num_layers as f32) * 0.5 + max_height;
    let fall_speed = 0.15;
    let pause_frames = 10;
    let frames_per_cycle =
        (pause_frames as f32) * (num_layers as f32 + 1.0) + total_dist / fall_speed;
    let total_cycles = (total_frames as f32 / frames_per_cycle).floor();
    let actual_frames_per_cycle = total_frames as f32 / total_cycles;
    let scaling_factor = frames_per_cycle / actual_frames_per_cycle;
    let mut scaled_frame = (frame as f32) * scaling_factor;
    let cycle = (scaled_frame / frames_per_cycle).floor();
    let color = saturated_color(cycle * 0.45);
    scaled_frame = scaled_frame % frames_per_cycle;

    let mut base_level = 0.0;
    let mut layer_level_min = 0.0;
    let mut layer_level_max = 0.0;
    for layer in 0..num_layers {
        let num_frames =
            (max_height - layer_height * (layer as f32)) / fall_speed + (pause_frames as f32);
        if scaled_frame < num_frames {
            layer_level_min = (max_height - scaled_frame * fall_speed).max(base_level);
            layer_level_max = layer_level_min + layer_height;
            scaled_frame = 0.0;
            break;
        } else {
            scaled_frame -= num_frames;
            base_level += layer_height;
        }
    }
    base_level -= scaled_frame * fall_speed;

    coords
        .iter()
        .map(|&coord| {
            if coord.2 < base_level || (coord.2 >= layer_level_min && coord.2 < layer_level_max) {
                color
            } else {
                (0.0, 0.0, 0.0)
            }
        })
        .collect()
}

fn fall_down_rainbow(coords: &[Coord], frame: usize, total_frames: usize) -> Vec<Color> {
    let max_height = coords.iter().map(|coord| coord.2).reduce(f32::max).unwrap();
    let num_layers = 8;
    let layer_height = max_height / (num_layers as f32);
    let total_dist = (max_height + layer_height) * (num_layers as f32) * 0.5 + max_height;
    let fall_speed = 0.15;
    let pause_frames = 10;
    let frames_per_cycle =
        (pause_frames as f32) * (num_layers as f32 + 1.0) + total_dist / fall_speed;
    let total_cycles = (total_frames as f32 / frames_per_cycle).floor();
    let actual_frames_per_cycle = total_frames as f32 / total_cycles;
    let scaling_factor = frames_per_cycle / actual_frames_per_cycle;
    let mut scaled_frame = (frame as f32) * scaling_factor;
    let cycle = (scaled_frame / frames_per_cycle).floor();
    let colors: Vec<_> = (0..num_layers)
        .map(|layer| saturated_color((layer as f32 + num_layers as f32 * cycle) * 0.45))
        .collect();
    scaled_frame = scaled_frame % frames_per_cycle;

    let mut base_level = 0.0;
    let mut layer_level_min = 0.0;
    let mut layer_level_max = 0.0;
    let mut current_layer = 0;
    for layer in 0..num_layers {
        let num_frames =
            (max_height - layer_height * (layer as f32)) / fall_speed + (pause_frames as f32);
        if scaled_frame < num_frames {
            current_layer = layer;
            layer_level_min = (max_height - scaled_frame * fall_speed).max(base_level);
            layer_level_max = layer_level_min + layer_height;
            scaled_frame = 0.0;
            break;
        } else {
            scaled_frame -= num_frames;
            base_level += layer_height;
        }
    }
    base_level -= scaled_frame * fall_speed;

    coords
        .iter()
        .map(|&coord| {
            if coord.2 < base_level {
                colors[(coord.2 / layer_height) as usize]
            } else if coord.2 >= layer_level_min && coord.2 < layer_level_max {
                colors[current_layer]
            } else {
                (0.0, 0.0, 0.0)
            }
        })
        .collect()
}

fn accelerate(coords: &[Coord], frame: usize, _total_frames: usize) -> Vec<Color> {
    let acceleration = 0.00002;
    let base_dist = acceleration * (frame as f32).powf(2.2);
    let max_height = coords.iter().map(|coord| coord.2).reduce(f32::max).unwrap();
    let level_height = max_height / 4.0;
    let double_height = level_height * 2.0;

    coords
        .iter()
        .map(|&coord| {
            let dist = base_dist + coord.2;
            let color_index = (dist / double_height) as usize;
            if dist % double_height < level_height {
                saturated_color(color_index as f32 * 0.45)
            } else {
                (0.0, 0.0, 0.0)
            }
        })
        .collect()
}

fn lerp(a: f32, b: f32, c: f32) -> f32 {
    a * (1.0 - c) + b * c
}

fn roll_around(coords: &[Coord], frame: usize, total_frames: usize) -> Vec<Color> {
    let frames_per_rotation = 60;
    let rotations_per_cycle = 8;
    let frames_per_cycle = frames_per_rotation * rotations_per_cycle;
    let num_cycles = total_frames / frames_per_cycle;
    let actual_frames = total_frames / num_cycles;
    let scaling_factor = actual_frames as f32 / (total_frames as f32);
    let scaled_frame = (frame as f32 * scaling_factor) % (frames_per_cycle as f32);
    let rotation_progress = scaled_frame / (frames_per_rotation as f32);
    let rotation_index = rotation_progress as usize;
    let lerp_factor = (rotation_progress.fract() * 2.0).min(1.0);
    let angle_values = [PI * 0.5, PI * 0.5, 0.0, 0.0, PI * -0.5, PI * -0.5, 0.0, 0.0];
    let z_angle_start = angle_values[rotation_index];
    let z_angle_end = angle_values[(rotation_index + 1) % 8];
    let x_angle_start = z_angle_end;
    let x_angle_end = angle_values[(rotation_index + 2) % 8];
    let z_angle = lerp(z_angle_start, z_angle_end, lerp_factor);
    let x_angle = lerp(x_angle_start, x_angle_end, lerp_factor);
    let max_height = coords.iter().map(|coord| coord.2).reduce(f32::max).unwrap();
    let z_offset = max_height / 2.0;
    let z_sc = z_angle.sin_cos();
    let x_sc = x_angle.sin_cos();

    coords
        .iter()
        .map(|&coord| {
            let coord = (coord.0, coord.1, coord.2 - z_offset);
            let coord = (
                coord.0 * z_sc.1 - coord.1 * z_sc.0,
                coord.0 * z_sc.0 + coord.1 * z_sc.1,
                coord.2,
            );
            let coord = (
                coord.0,
                coord.1 * x_sc.1 - coord.2 * x_sc.0,
                coord.1 * x_sc.0 + coord.2 * x_sc.1,
            );
            let mut quadrant = 0;
            if coord.0 > 0.0 {
                quadrant += 1;
            }
            if coord.1 > 0.0 {
                quadrant += 2;
            }
            if coord.2 > 0.0 {
                quadrant += 4;
            }
            saturated_color(quadrant as f32 * 0.45)
        })
        .collect()
}

fn twinkle(coords: &[Coord], frame: usize, total_frames: usize) -> Vec<Color> {
    let num_phases = 4;
    let mut phases: Vec<_> = (0..coords.len()).map(|i| i % num_phases).collect();
    let mut rng = StdRng::seed_from_u64(42);
    phases.shuffle(&mut rng);

    let angle = frame as f32 * PI * 6.0 / (total_frames as f32);

    phases
        .into_iter()
        .map(|phase| {
            let phase_color = saturated_color(phase as f32 * 0.3);
            let phase_angle = (phase as f32 * PI * 2.0 / (num_phases as f32)) - angle;
            let brightness = phase_angle.sin().max(0.0);
            (
                phase_color.0 * brightness,
                phase_color.1 * brightness,
                phase_color.2 * brightness,
            )
        })
        .collect()
}
