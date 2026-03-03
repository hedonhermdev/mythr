#![allow(dead_code)]
use image::{GrayImage, Luma, Pixel, Rgb, RgbImage, imageops::flip_vertical_in_place};
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
    let xmin = *[a.x, b.x, c.x].iter().min().unwrap();
    let ymin = *[a.y, b.y, c.y].iter().min().unwrap();

    let xmax = *[a.x, b.x, c.x].iter().max().unwrap();
    let ymax = *[a.y, b.y, c.y].iter().max().unwrap();

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

    if area_abc == 0.0 {
        return;
    }

    let buf = img.as_mut();

    let zbuf = z_buffer.as_mut();

    let rgb_stride = WIDTH as usize * Rgb::<u8>::CHANNEL_COUNT as usize * size_of::<u8>();

    let gray_stride = WIDTH as usize * Luma::<u8>::CHANNEL_COUNT as usize * size_of::<u8>();

    buf.par_chunks_mut(rgb_stride)
        .zip(zbuf.par_chunks_mut(gray_stride))
        .enumerate()
        .for_each(|(y, (img_row, zbuf_row))| {
            if (bbmin.y..bbmax.y).contains(&(y as u32)) {
                for x in bbmin.x..bbmax.x {
                    let y = y as u32;
                    let p = Coord::new(x, y);

                    let alpha = signed_area(p, b, c) / area_abc;
                    let beta = signed_area(a, p, c) / area_abc;
                    let gamma = signed_area(a, b, p) / area_abc;

                    let z = alpha * za + beta * zb + gamma * zc;

                    let z_idx = x as usize;

                    if alpha >= 0.0 && gamma >= 0.0 && beta >= 0.0 && z >= zbuf_row[z_idx] {
                        let img_idx = (x * 3) as usize;
                        img_row[img_idx..img_idx + 3].copy_from_slice(&color.0);
                        zbuf_row[z_idx] = z;
                    }
                }
            }
        });
}

fn project_transform_scale(v: &Vertex) -> Vertex {
    // orthogonal projection
    // front view (looking down z-axis)
    // [-1, 1] -> [0, 2]
    let v = (v.x + 1.0, v.y + 1.0, v.z + 1.0);

    // [0, 2] -> [0, W], [0, 2] -> [0, H]
    let v = (
        v.0 * (WIDTH - 1) as f32 / 2.0,
        v.1 * (HEIGHT - 1) as f32 / 2.0,
        v.2 * (255.0) / 2.0,
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
    let mut z_buffer = vec![0.0; (WIDTH * HEIGHT) as usize];

    for [a, b, c] in wavefront
        .triangles()
        .map(|t| t.map(|v| project_transform_scale(&rot(v))))
    {
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
    let mut z_buffer = GrayImage::new(WIDTH, HEIGHT);

    draw_wavefront(&mut img, &wavefront);

    // because the tutorial uses a different coordinate system than ours
    flip_vertical_in_place(&mut img);
    img.save("out.png")?;

    flip_vertical_in_place(&mut z_buffer);
    z_buffer.save("zbuffer.png")?;

    Ok(())
}
