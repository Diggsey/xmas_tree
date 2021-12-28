use std::{error::Error, io::stdout, path::PathBuf};

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

type EffectFn = fn(&[Coord], usize) -> Vec<Color>;

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
        other => {
            println!("Unknown effect: {}", other);
            return Ok(());
        }
    };
    for frame in 0..opt.len {
        sequence_csv.write_field(frame.to_string())?;
        sequence_csv.write_record(
            effect_fn(&coords, frame)
                .into_iter()
                .flat_map(|color| [color.0, color.1, color.2])
                .map(|v| ((v * 255.0) as i32).to_string()),
        )?;
    }

    Ok(())
}

fn barber_pole(coords: &[Coord], frame: usize) -> Vec<Color> {
    let offset = frame as f32 * 0.05;
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
