---
date: 03/03/2026
crate: crates/tinyrenderer
---
---
**Goals:**
- continue with tinyrenderer
- revisit matrices
- rotation and projection
---

For this chapter, I've refactored the code to use primitives from the `nalgebra` crate. 

First we'll try a basic rotation using a rotation matrix. 

As an example, let's rotate the figure by 30 degrees along the y axis. 

```rust
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
```

```rust
fn draw_wavefront(img: &mut RgbImage, wavefront: &Wavefront) {
    for [a, b, c] in wavefront.triangles() {
        let color: Rgb<u8> = Rgb(rand::random());
        let a = project_transform_scale(&rot(a));
        let b = project_transform_scale(&rot(b));
        let c = project_transform_scale(&rot(c));

        triangle(img, a, b, c, color);
    }
}
```



| ![rotation-1.png](./media/rotation-1.png) | ![rotation-2.png](./media/rotation-2.png) |
| ------------------- | ------------------- |
