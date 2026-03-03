#![allow(dead_code)]
use image::{Pixel, Rgb, RgbImage, imageops::flip_vertical_in_place};
use nalgebra::{Matrix3, Point2};
use rayon::prelude::*;
use std::{f32::consts::PI, path::PathBuf};

use wavefront::{Vertex, Wavefront};

mod wavefront;

const WHITE: Rgb<u8> = Rgb([255, 255, 255]);
const RED: Rgb<u8> = Rgb([255, 0, 0]);
const GREEN: Rgb<u8> = Rgb([0, 255, 0]);
const BLUE: Rgb<u8> = Rgb([64, 128, 255]);
const YELLOW: Rgb<u8> = Rgb([255, 200, 0]);

const HEIGHT: u32 = 800;
const WIDTH: u32 = 800;

type Coord = Point2<u32>;

fn bbox(a: Coord, b: Coord, c: Coord) -> (Coord, Coord) {
    let xmin = (*[a.x, b.x, c.x].iter().min().unwrap()).clamp(0, WIDTH - 1);
    let ymin = (*[a.y, b.y, c.y].iter().min().unwrap()).clamp(0, HEIGHT - 1);

    let xmax = (*[a.x, b.x, c.x].iter().max().unwrap()).clamp(0, WIDTH - 1);
    let ymax = (*[a.y, b.y, c.y].iter().max().unwrap()).clamp(0, HEIGHT - 1);

    (Coord::new(xmin, ymin), Coord::new(xmax, ymax))
}

fn signed_area(a: Coord, b: Coord, c: Coord) -> f32 {
    let [a, b, c] = [a, b, c].map(|p| p.cast::<f32>());

    0.5 * ((b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x))
}

fn triangle(
    img: &mut RgbImage,
    z_buffer: &mut [f32],
    a: Vertex,
    b: Vertex,
    c: Vertex,
    color: Rgb<u8>,
) {
    let [za, zb, zc] = [a, b, c].map(|v| v[2]); // z-coords

    let [a, b, c] = [a, b, c].map(|v| Coord::new(v[0] as u32, v[1] as u32));

    let (bbmin, bbmax) = bbox(a, b, c);

    let area_abc = signed_area(a, b, c);

    // if triangle is degenerate
    if area_abc == 0.0 {
        return;
    }

    let inv_area = 1.0 / area_abc;

    let buf = img.as_mut();

    let img_stride = WIDTH as usize * Rgb::<u8>::CHANNEL_COUNT as usize;

    buf.par_chunks_mut(img_stride)
        .zip(z_buffer.par_chunks_mut(WIDTH as usize))
        .enumerate()
        .for_each(|(y, (img_row, zbuf_row))| {
            let y = y as u32;

            if y < bbmin.y || y > bbmax.y {
                return;
            }

            for x in bbmin.x..=bbmax.x {
                let p = Coord::new(x, y);

                let alpha = signed_area(p, b, c) * inv_area;
                let beta = signed_area(a, p, c) * inv_area;
                let gamma = signed_area(a, b, p) * inv_area;

                let z = alpha * za + beta * zb + gamma * zc;

                let z_idx = x as usize;

                if alpha >= 0.0 && gamma >= 0.0 && beta >= 0.0 && z >= zbuf_row[z_idx] {
                    let img_idx = (x * 3) as usize;
                    img_row[img_idx..img_idx + 3].copy_from_slice(&color.0);
                    zbuf_row[z_idx] = z;
                }
            }
        });
}

fn project(v: &Vertex) -> Vertex {
    // orthogonal projection
    // front view (looking down z-axis)
    // [-1, 1] -> [0, 2]
    let v = (v.x + 1.0, v.y + 1.0, v.z + 1.0);

    // [0, 2] -> [0, W], [0, 2] -> [0, H]
    let v = (
        v.0 * (WIDTH - 1) as f32 / 2.0,
        v.1 * (HEIGHT - 1) as f32 / 2.0,
        v.2,
    );

    Vertex::new(v.0, v.1, v.2)
}

fn rot(v: &Vertex) -> Vertex {
    let angle = PI / 6.0;

    #[rustfmt::skip]
    let mat = Matrix3::new(
        f32::cos(angle), 0.0, f32::sin(angle),
        0.0, 1.0, 0.0,
        -f32::sin(angle), 0.0, f32::cos(angle),
    );

    mat * v
}

fn persp(v: &Vertex) -> Vertex {
    let c = 3.0;

    v / (1.0 - (v.z / c))
}

fn draw_wavefront(img: &mut RgbImage, wavefront: &Wavefront) {
    let mut z_buffer = vec![-f32::INFINITY; (WIDTH * HEIGHT) as usize];

    for [a, b, c] in wavefront.triangles().map(|t| t.map(|v| project(&rot(v)))) {
        let color: Rgb<u8> = Rgb(rand::random());

        triangle(img, &mut z_buffer, a, b, c, color);
    }
}

fn main() -> anyhow::Result<()> {
    let path: PathBuf = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Usage: tinyrenderer <path_to_obj_file>"))?
        .into();

    let wavefront = Wavefront::read_from_file(&path)?;

    let mut img = RgbImage::new(WIDTH, HEIGHT);

    draw_wavefront(&mut img, &wavefront);

    // because the tutorial uses a different coordinate system than ours
    flip_vertical_in_place(&mut img);
    img.save("out.png")?;

    Ok(())
}
