use clap::Parser;
use image::{ImageBuffer, Rgba};
use palette::Clamp;
use palette::FromColor;
use palette::{LinSrgb, Srgb};
use rayon::prelude::*;
use tempergb::rgb_from_temperature;

#[derive(clap::ValueEnum, Clone, Debug)]
enum ObjectFit {
    Contain,
    Cover,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(index = 1)]
    output: String,
    #[clap(long, default_value_t = 800)]
    width: u32,
    #[clap(long, default_value_t = 600)]
    height: u32,
    #[clap(long, default_value_t = 6500)]
    temperature: u32,
    #[clap(long, default_value_t = 1.0)]
    intensity: f32,

    #[clap(long, default_value_t = 0.0)]
    light_pos_x: f32,
    #[clap(long, default_value_t = 1.0)]
    light_pos_y: f32,
    #[clap(long, default_value_t = 1.0)]
    light_pos_z: f32,

    #[clap(long, default_value_t = 0.0)]
    light_dir_x: f32,
    #[clap(long, default_value_t = -1.0)]
    light_dir_y: f32,
    #[clap(long, default_value_t = 0.0)]
    light_dir_z: f32,

    #[clap(long, default_value_t = 90.0)]
    outer_angle: f32,
    #[clap(long, default_value_t = 0.8)]
    inner_angle_factor: f32,
    #[clap(long, default_value = "contain")]
    object_fit: ObjectFit,

    #[clap(long, default_value_t = 0.0)]
    dithering: f32,
}

fn main() {
    let args = Args::parse();
    let Args {
        dithering,
        height,
        inner_angle_factor,
        intensity,
        outer_angle,
        output,
        temperature,
        width,
        ..
    } = args;
    let light_pos = (args.light_pos_x, args.light_pos_y, args.light_pos_z);
    let light_dir = {
        let len =
            (args.light_dir_x.powi(2) + args.light_dir_y.powi(2) + args.light_dir_z.powi(2)).sqrt();
        (
            args.light_dir_x / len,
            args.light_dir_y / len,
            args.light_dir_z / len,
        )
    };
    let outer_angle = f32::to_radians(outer_angle);
    let inner_angle = outer_angle * inner_angle_factor;
    let temp_color = {
        let rgb = rgb_from_temperature(temperature);
        Srgb::from_components(rgb.into_components()).into_format::<f32>()
    };

    let inner_cos = inner_angle.cos();
    let outer_cos = outer_angle.cos();

    let mut image = ImageBuffer::<Rgba<f32>, Vec<f32>>::new(width, height);
    let aspect_ratio = width as f32 / height as f32;
    image.par_chunks_mut(4).enumerate().for_each(|(i, pixel)| {
        let x_idx = (i as u32) % width;
        let y_idx = (i as u32) / width;
        let (x, y) = {
            let (x, y) = (
                x_idx as f32 / width as f32 * 2.0 - 1.0,
                y_idx as f32 / height as f32 * -2.0 + 1.0,
            );
            if matches!(args.object_fit, ObjectFit::Contain) == (aspect_ratio >= 1.0) {
                (x * aspect_ratio, y)
            } else {
                (x, y / aspect_ratio)
            }
        };
        let z = 0.0;

        let dithering = (rand(x, y) * 2.0 - 1.0) * dithering;

        let (dx, dy, dz) = (x - light_pos.0, y - light_pos.1, z - light_pos.2);
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();
        let (lx, ly, lz) = (-dx / distance, -dy / distance, -dz / distance);

        let cos = -lx * light_dir.0 + -ly * light_dir.1 + -lz * light_dir.2;
        let spot_effect = smoothstep(outer_cos, inner_cos, cos);

        let (nx, ny, nz) = (0.0, 0.0, 1.0);
        let intensity = intensity / (distance * distance + f32::EPSILON) * spot_effect;
        let diffusion = intensity * (lx * nx + ly * ny + lz * nz).max(0.0);
        let lin_color = LinSrgb::from_color(temp_color) * diffusion + dithering;

        let final_color: Srgb<f32> = Srgb::from_linear(lin_color).clamp();
        let (r, g, b) = final_color.into_components();
        (pixel[0], pixel[1], pixel[2], pixel[3]) = (r, g, b, 1.0);
    });
    image
        .save(output)
        .expect("Failed to save the generated image");
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn rand(x: f32, y: f32) -> f32 {
    return ((x * 12.9898 + y * 78.233) * 43758.5453123)
        .sin()
        .rem_euclid(1.0);
}
