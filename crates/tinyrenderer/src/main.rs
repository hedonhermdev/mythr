#![allow(dead_code)]
use image::{Pixel, Rgb, RgbImage, imageops::flip_vertical_in_place};
use nalgebra::Matrix3;
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

type Coord = (u32, u32);

fn find_bbox(a: Coord, b: Coord, c: Coord) -> (Coord, Coord) {
    let xmin = *[a.0, b.0, c.0].iter().min().unwrap();
    let ymin = *[a.1, b.1, c.1].iter().min().unwrap();

    let xmax = *[a.0, b.0, c.0].iter().max().unwrap();
    let ymax = *[a.1, b.1, c.1].iter().max().unwrap();

    ((xmin, ymin), (xmax, ymax))
}

fn draw_bbox(img: &mut RgbImage, bbmin: Coord, bbmax: Coord, color: Rgb<u8>) {
    for x in bbmin.0..bbmax.0 {
        for y in bbmin.1..bbmax.1 {
            img[(x, y)] = color;
        }
    }
}

fn signed_area(a: Coord, b: Coord, c: Coord) -> f32 {
    let [(ax, ay), (bx, by), (cx, cy)] = [a, b, c].map(|(x, y)| (x as f32, y as f32));

    0.5 * ((bx - ax) * (cy - ay) - (by - ay) * (cx - ax))
}

fn triangle(img: &mut RgbImage, a: Coord, b: Coord, c: Coord, color: Rgb<u8>) {
    let (bbmin, bbmax) = find_bbox(a, b, c);

    let area_abc = signed_area(a, b, c);

    if area_abc < 1.0 {
        return;
    }

    let buf = img.as_mut();

    let stride = WIDTH as usize * Rgb::<u8>::CHANNEL_COUNT as usize * size_of::<u8>();

    buf.par_chunks_mut(stride).enumerate().for_each(|(y, row)| {
        if (bbmin.1..bbmax.1).contains(&(y as u32)) {
            for x in bbmin.0..bbmax.0 {
                let y = y as u32;
                let p = (x, y);

                let alpha = signed_area(p, b, c) / area_abc;
                let beta = signed_area(a, p, c) / area_abc;
                let gamma = signed_area(a, b, p) / area_abc;

                if alpha >= 0.0 && gamma >= 0.0 && beta >= 0.0 {
                    let idx = (x * 3) as usize;
                    row[idx..idx + 3].copy_from_slice(&color.0);
                }
            }
        }
    });
}

fn rot(v: &Vertex) -> Vertex {
    let angle = PI / 6.0;

    let mat = Matrix3::new(
        f32::cos(angle),
        0.0,
        f32::sin(angle),
        0.0,
        1.0,
        0.0,
        -f32::sin(angle),
        0.0,
        f32::cos(angle),
    );

    mat * v
}

fn project_transform_scale(v: &Vertex) -> Coord {
    // orthogonal projection
    // front view (looking down z-axis)
    let p = (v[0], v[1]);

    // [-1, 1] -> [0, 2]
    let p = (p.0 + 1.0, p.1 + 1.0);

    // [0, 2] -> [0, W], [0, 2] -> [0, H]
    let p = (
        p.0 * (WIDTH - 1) as f32 / 2.0,
        p.1 * (HEIGHT - 1) as f32 / 2.0,
    );

    (p.0.round() as u32, p.1.round() as u32)
}

fn draw_wavefront(img: &mut RgbImage, wavefront: &Wavefront) {
    for [a, b, c] in wavefront.triangles() {
        let color: Rgb<u8> = Rgb(rand::random());
        let a = project_transform_scale(&rot(a));
        let b = project_transform_scale(&rot(b));
        let c = project_transform_scale(&rot(c));

        triangle(img, a, b, c, color);
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
