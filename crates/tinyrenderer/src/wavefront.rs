use std::{fs::File, io::Read, path::Path};

use anyhow::Context;

pub type Vertex3 = (f32, f32, f32);
pub type Triangle = (usize, usize, usize);

pub struct Wavefront {
    vertices: Vec<Vertex3>,
    triangles: Vec<Triangle>,
}

impl Wavefront {
    fn new(vertices: Vec<Vertex3>, triangles: Vec<Triangle>) -> Self {
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

    pub fn vertices(&self) -> &[(f32, f32, f32)] {
        &self.vertices
    }

    pub fn triangles(&self) -> &[(usize, usize, usize)] {
        &self.triangles
    }
}

fn parse_vertex_line(l: &str) -> anyhow::Result<Vertex3> {
    let mut iter = l.split(" ");

    iter.next();

    let x = iter.next().context("expected x coord")?.parse::<f32>()?;

    let y = iter.next().context("expected y coord")?.parse::<f32>()?;

    let z = iter.next().context("expected z coord")?.parse::<f32>()?;

    Ok((x, y, z))
}

fn parse_face_line(l: &str) -> anyhow::Result<Triangle> {
    let mut iter = l.split(" ");

    iter.next();

    let a = iter
        .next()
        .context("expected vertex")?
        .split("/")
        .next()
        .context("expected index")?
        .parse::<usize>()?;

    let b = iter
        .next()
        .context("expected vertex")?
        .split("/")
        .next()
        .context("expected index")?
        .parse::<usize>()?;

    let c = iter
        .next()
        .context("expected vertex")?
        .split("/")
        .next()
        .context("expected index")?
        .parse::<usize>()?;

    Ok((a, b, c))
}
