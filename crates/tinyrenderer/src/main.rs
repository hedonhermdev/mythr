#![allow(dead_code)]
use image::{Pixel, Rgb, RgbImage, imageops::flip_vertical_in_place};
use nalgebra::{Matrix3, Matrix4, Point2, Vector, Vector3, Vector4};
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

fn rasterize(
    img: &mut RgbImage,
    z_buffer: &mut [f32],
    viewport: Matrix4<f32>,
    clip: [Vector4<f32>; 3],
    color: Rgb<u8>,
) {
    // normalized device coordinates [x, y, z, w] -> [x/w, y/w, z/w, 1]
    let ndc = clip.map(|c| c / c.w);

    let screen = ndc.map(|n| (viewport * n).xy());

    #[rustfmt::skip]
    let abc = Matrix3::new(
        screen[0].x, screen[0].y, 1.,
        screen[1].x, screen[1].y, 1.,
        screen[2].x, screen[2].y, 1.,
    );

    if abc.determinant() < 1. {
        return;
    }

    let bbmin_x = screen[0].x.min(screen[1].x).min(screen[2].x).round() as u32;
    let bbmin_y = screen[0].y.min(screen[1].y).min(screen[2].y).round() as u32;

    let bbmax_x = screen[0].x.max(screen[1].x).max(screen[2].x).round() as u32;
    let bbmax_y = screen[0].y.max(screen[1].y).max(screen[2].y).round() as u32;

    let buf = img.as_mut();

    let img_stride = WIDTH as usize * Rgb::<u8>::CHANNEL_COUNT as usize;

    buf.par_chunks_mut(img_stride)
        .zip(z_buffer.par_chunks_mut(WIDTH as usize))
        .enumerate()
        .for_each(|(y, (img_row, zbuf_row))| {
            let y = y as u32;

            if y < bbmin_y || y > bbmax_y {
                return;
            }

            for x in bbmin_x..=bbmax_x {
                let v = Vector3::new(x as f32, y as f32, 1.);

                let bc: Vector3<f32> = abc.try_inverse().unwrap().transpose() * v;

                if bc.x < 0. || bc.y < 0. || bc.z < 0. {
                    continue;
                }

                let z = bc.dot(&Vector3::new(ndc[0].z, ndc[1].z, ndc[2].z));

                let z_idx = x as usize;

                if z < zbuf_row[z_idx] {
                    continue;
                }

                let img_idx = (x * 3) as usize;
                img_row[img_idx..img_idx + 3].copy_from_slice(&color.0);
                zbuf_row[z_idx] = z;
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

fn viewport(x: u32, y: u32, w: u32, h: u32) -> Matrix4<f32> {
    let x = x as f32;
    let y = y as f32;
    let w = w as f32;
    let h = h as f32;

    #[rustfmt::skip]
    let viewport = Matrix4::new(
        w/2., 0., 0., x + w/2.,
        0., h/2., 0., y + h/2.,
        0., 0., 1., 0.,
        0.,0., 0., 1.
    );

    viewport
}

fn perspective(f: f32) -> Matrix4<f32> {
    #[rustfmt::skip]
    let perspective = Matrix4::new(
        1., 0., 0., 0.,
        0., 1., 0., 0.,
        0., 0., 0., 1.,
        0., 0., -1./f, 1.
    );

    perspective
}

fn look_at(eye: Vector3<f32>, center: Vector3<f32>, up: Vector3<f32>) -> Matrix4<f32> {
    let n: Vector3<f32> = (eye - center).normalize();

    let l = up.cross(&n).normalize();

    let m = n.cross(&l).normalize();

    #[rustfmt::skip]
    let rotation = Matrix4::new(
        l.x, l.y, l.z, 0.,
        m.x, m.y, m.z, 0.,
        n.x, n.y, n.z, 0.,
        0., 0., 0., 1.
    );

    #[rustfmt::skip]
    let translation = Matrix4::new(
        1., 0., 0., -eye.x,
        0., 1., 0., -eye.y,
        0., 0., 1., -eye.z,
        0., 0., 0., 1.,
    );

    rotation * translation
}

fn render(img: &mut RgbImage, wavefront: &Wavefront) {
    let mut z_buffer = vec![-f32::INFINITY; (WIDTH * HEIGHT) as usize];

    let eye = Vector3::new(-1., 0., 2.);
    let center = Vector3::new(0., 0., 0.);
    let up = Vector3::new(0., 1., 0.);

    let model_view = look_at(eye, center, up);
    let perspective = perspective((eye - center).norm());

    let viewport = viewport(WIDTH / 16, HEIGHT / 16, WIDTH * 7 / 8, HEIGHT * 7 / 8);

    for clip in wavefront.triangles().map(|t| {
        t.map(|v| {
            let v = Vector4::new(v.x, v.y, v.z, 1.0);

            perspective * model_view * v
        })
    }) {
        let color: Rgb<u8> = Rgb(rand::random());

        rasterize(img, &mut z_buffer, viewport, clip, color);
    }
}

fn main() -> anyhow::Result<()> {
    let path: PathBuf = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Usage: tinyrenderer <path_to_obj_file>"))?
        .into();

    let wavefront = Wavefront::read_from_file(&path)?;

    let mut img = RgbImage::new(WIDTH, HEIGHT);

    render(&mut img, &wavefront);

    // because the tutorial uses a different coordinate system than ours
    flip_vertical_in_place(&mut img);
    img.save("out.png")?;

    Ok(())
}
