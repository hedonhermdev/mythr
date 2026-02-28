#![allow(dead_code)]
use image::{Rgb, RgbImage, imageops::flip_vertical_in_place};
use std::{mem::swap, path::PathBuf};

use crate::wavefront::{Vertex3, Wavefront};

mod wavefront;

const WHITE: Rgb<u8> = Rgb([255, 255, 255]);
const RED: Rgb<u8> = Rgb([255, 0, 0]);
const GREEN: Rgb<u8> = Rgb([0, 255, 0]);
const BLUE: Rgb<u8> = Rgb([64, 128, 255]);
const YELLOW: Rgb<u8> = Rgb([255, 200, 0]);

const HEIGHT: u32 = 512;
const WIDTH: u32 = 512;

type Point = (u32, u32);

#[inline(always)]
fn point(img: &mut RgbImage, p: Point, color: Rgb<u8>) {
    img[p] = color;
}

#[inline(always)]
fn line(img: &mut RgbImage, mut a: Point, mut b: Point, color: Rgb<u8>) {
    let is_steeper = a.0.abs_diff(b.0) < a.1.abs_diff(b.1);

    if is_steeper {
        // transpose
        a = (a.1, a.0);
        b = (b.1, b.0);
    }

    if a.0 > b.0 {
        swap(&mut a, &mut b);
    }

    let dx = (b.0 - a.0) as i32;
    let dy = (b.1 as i32 - a.1 as i32).abs();

    let mut error = dx / 2;
    let ystep = if a.1 < b.1 { 1 } else { -1 };

    let mut y = a.1 as i32;

    for x in a.0..=b.0 {
        if is_steeper {
            img[(y as u32, x)] = color;
        } else {
            img[(x, y as u32)] = color;
        }

        error -= dy;
        if error < 0 {
            y += ystep;
            error += dx;
        }
    }
}

fn project_transform_scale(v: &Vertex3) -> Point {
    // orthogonal projection
    // front view (looking down z-axis)
    let p = (v.0, v.1);

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
    for vertex in wavefront.vertices() {
        let p = project_transform_scale(vertex);

        point(img, p, WHITE);
    }

    let vertices = wavefront.vertices();

    for triangle in wavefront.triangles() {
        let a = project_transform_scale(&vertices[triangle.0 - 1]);
        let b = project_transform_scale(&vertices[triangle.1 - 1]);
        let c = project_transform_scale(&vertices[triangle.2 - 1]);

        line(img, a, b, RED);
        line(img, b, c, RED);
        line(img, c, a, RED);
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
