use std::{fs::File, io::Read, path::Path, slice::Iter};

use anyhow::Context;
use image::write_buffer_with_format;
use nalgebra::Point3;

pub type Vertex = Point3<f32>;
pub type Triangle = [usize; 3];

pub struct Wavefront {
    vertices: Vec<Vertex>,
    triangles: Vec<Triangle>,
}

impl Wavefront {
    fn new(vertices: Vec<Vertex>, triangles: Vec<Triangle>) -> Self {
        Self {
            vertices,
            triangles,
        }
    }

    pub fn read_from_file(p: &Path) -> anyhow::Result<Self> {
        let mut file = File::open(p)?;
        let mut buf = String::new();

        file.read_to_string(&mut buf)?;

        let mut vertices = vec![];
        let mut triangles = vec![];

        for line in buf.lines() {
            if line.starts_with("v ") {
                let v = parse_vertex_line(line)?;

                vertices.push(v);
            }

            if line.starts_with("f ") {
                let t = parse_face_line(line)?;

                triangles.push(t);
            }
        }

        Ok(Self {
            vertices,
            triangles,
        })
    }

    pub fn vertices(&self) -> impl Iterator<Item = &Vertex> {
        self.vertices.iter()
    }

    pub fn triangles(&self) -> impl Iterator<Item = [&Vertex; 3]> {
        self.triangles.iter().map(|triangle| {
            let [a, b, c] = triangle;

            // wavefront triangles are 1-indexed
            [
                &self.vertices[*a - 1],
                &self.vertices[*b - 1],
                &self.vertices[*c - 1],
            ]
        })
    }
}

fn parse_vertex_line(l: &str) -> anyhow::Result<Vertex> {
    let [x, y, z] = l
        .split_whitespace()
        .skip(1)
        .map(|v| v.parse::<f32>().context("expected vertex coord"))
        .collect::<anyhow::Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected 3 coords"))?;

    Ok(Point3::new(x, y, z))
}

fn parse_face_line(l: &str) -> anyhow::Result<[usize; 3]> {
    let indices = l
        .split_whitespace()
        .skip(1) // skip the "f " prefix
        .map(|v| {
            v.split('/')
                .next()
                .context("expected index")?
                .parse::<usize>()
                .map_err(Into::into)
        })
        .collect::<Result<Vec<_>, anyhow::Error>>()?;

    indices
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected exactly 3 vertices"))
}
