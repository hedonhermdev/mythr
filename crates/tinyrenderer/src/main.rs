#![allow(dead_code)]
use image::{Rgb, RgbImage, imageops::flip_vertical_in_place};
use std::{mem::swap, path::PathBuf};

const WHITE: Rgb<u8> = Rgb([255, 255, 255]);
const RED: Rgb<u8> = Rgb([255, 0, 0]);
const GREEN: Rgb<u8> = Rgb([0, 255, 0]);
const BLUE: Rgb<u8> = Rgb([64, 128, 255]);
const YELLOW: Rgb<u8> = Rgb([255, 200, 0]);

const HEIGHT: u32 = 128;
const WIDTH: u32 = 128;

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

fn triangle(img: &mut RgbImage, mut a: Point, mut b: Point, mut c: Point, color: Rgb<u8>) {
    if a.1 > b.1 {
        swap(&mut a, &mut b);
    }
    if a.1 > c.1 {
        swap(&mut a, &mut c);
    }

    if b.1 > c.1 {
        swap(&mut b, &mut c);
    }

    let ax = a.0 as i32;
    let ay = a.1 as i32;

    let bx = b.0 as i32;
    let by = b.1 as i32;

    let cx = c.0 as i32;
    let cy = c.1 as i32;

    if a.1 != b.1 {
        for y in a.1..=b.1 {
            let x1 = ax + (y as i32 - ay) * (cx - ax) / (cy - ay);
            let x2 = ax + (y as i32 - ay) * (bx - ax) / (by - ay);

            for x in x1.min(x2)..x1.max(x2) {
                img[(x as u32, y)] = color;
            }
        }
    }

    if c.1 != b.1 {
        for y in b.1..=c.1 {
            let x1 = ax + (y as i32 - ay) * (cx - ax) / (cy - ay);
            let x2 = bx + (y as i32 - by) * (cx - bx) / (cy - by);

            for x in x1.min(x2)..x1.max(x2) {
                img[(x as u32, y)] = color;
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut img = RgbImage::new(WIDTH, HEIGHT);

    triangle(&mut img, (7, 45), (35, 100), (45, 60), RED);
    triangle(&mut img, (120, 35), (90, 5), (45, 110), WHITE);
    triangle(&mut img, (115, 83), (80, 90), (85, 120), GREEN);

    // because the tutorial uses a different coordinate system than ours
    flip_vertical_in_place(&mut img);
    img.save("out.png")?;

    Ok(())
}
