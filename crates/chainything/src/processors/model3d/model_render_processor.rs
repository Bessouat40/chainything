use crate::processors::base_processor::{Processor, ProcessorError};
use crate::processors::images::greyscale_processor::RawImage;
use crate::processors::model3d::mesh::Mesh3D;
use std::sync::Arc;

/// Default side length (pixels) used when no resolution is supplied.
const DEFAULT_RESOLUTION: u32 = 512;
/// Fraction of the image kept empty around the model on each side.
const MARGIN: f32 = 0.1;
/// Fixed view rotation (radians) giving a pleasant three-quarter angle.
const ROT_X: f32 = 0.5;
const ROT_Y: f32 = 0.6;
/// Ambient term so faces turned away from the light are never pure black.
const AMBIENT: f32 = 0.25;

/// Renders a [`Mesh3D`] to a 2D [`RawImage`] using a small built-in software
/// rasterizer (no GPU, no extra dependencies).
///
/// The mesh is rotated to a fixed three-quarter view, projected orthographically
/// to fit the output square, then rasterized with a depth (z) buffer and
/// two-sided Lambert shading from a fixed light. The result is an RGB image,
/// ready to feed an `ImageDisplay` or `ImageSave` node.
///
/// ### Input
/// 1. `Arc<Mesh3D>` — the mesh to render.
/// 2. *(optional)* the square resolution in pixels, accepted as `Arc<u32>` or
///    `Arc<String>`. Defaults to `512` when absent.
///
/// ### Output
/// * One `Arc<RawImage>` (RGB, `resolution × resolution`).
///
/// ### Errors
/// * [`ProcessorError::MissingInput`] if no mesh is provided.
/// * [`ProcessorError::InvalidInput`] if an input has an unexpected type.
pub struct ModelRenderProcessor {
    id: String,
    input: Option<Arc<Mesh3D>>,
    resolution: Option<u32>,
    output: Option<Arc<RawImage>>,
}

impl ModelRenderProcessor {
    pub fn new(id: String) -> Self {
        ModelRenderProcessor {
            id,
            input: None,
            resolution: None,
            output: None,
        }
    }
}

/// A vertex after view rotation: screen-space `x`/`y` plus a `depth` used by the
/// z-buffer (larger is nearer the viewer).
#[derive(Clone, Copy)]
struct Projected {
    x: f32,
    y: f32,
    depth: f32,
    /// Rotated 3D position, kept to compute face normals for shading.
    world: [f32; 3],
}

/// Signed area (×2) of the triangle `a, b, c` in screen space.
fn edge(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> f32 {
    (c.0 - a.0) * (b.1 - a.1) - (c.1 - a.1) * (b.0 - a.0)
}

impl Processor for ModelRenderProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(
        &mut self,
        mut inputs: Vec<Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<(), ProcessorError> {
        if inputs.is_empty() {
            return Err(ProcessorError::MissingInput(format!(
                "Processor {} requires at least 1 input (mesh), got 0",
                self.id()
            )));
        }

        let mesh_input = inputs.remove(0);
        self.input = Some(mesh_input.downcast::<Mesh3D>().map_err(|_| {
            ProcessorError::InvalidInput(format!(
                "Invalid input type (expected Mesh3D) for processor {}",
                self.id()
            ))
        })?);

        if !inputs.is_empty() {
            let res_input = inputs.remove(0);
            if let Ok(r) = res_input.clone().downcast::<u32>() {
                self.resolution = Some(*r);
            } else if let Ok(s) = res_input.downcast::<String>() {
                let parsed: u32 = s.trim().parse().map_err(|_| {
                    ProcessorError::InvalidInput(format!(
                        "Cannot parse resolution as u32 for processor {}",
                        self.id()
                    ))
                })?;
                self.resolution = Some(parsed);
            } else {
                return Err(ProcessorError::InvalidInput(format!(
                    "Invalid input type for resolution (expected u32 or String) for processor {}",
                    self.id()
                )));
            }
        }

        Ok(())
    }

    fn get_output(&self) -> Vec<Arc<dyn std::any::Any + Send + Sync>> {
        self.output
            .clone()
            .into_iter()
            .map(|out| out as Arc<dyn std::any::Any + Send + Sync>)
            .collect()
    }

    fn process(&mut self) -> Result<(), ProcessorError> {
        let mesh = self.input.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput(format!("Missing mesh input for processor {}", self.id()))
        })?;

        let res = self.resolution.unwrap_or(DEFAULT_RESOLUTION).max(1);
        let width = res as usize;
        let height = res as usize;

        // Background (dark blue-grey) and the model's base colour.
        let bg = [30u8, 30u8, 40u8];
        let base = [205.0f32, 205.0f32, 215.0f32];

        let mut pixels = Vec::with_capacity(width * height * 3);
        for _ in 0..width * height {
            pixels.extend_from_slice(&bg);
        }

        // Nothing to draw — return the empty background.
        if mesh.vertices.is_empty() || mesh.faces.is_empty() {
            self.output = Some(Arc::new(RawImage {
                width: res,
                height: res,
                pixels,
            }));
            return Ok(());
        }

        // Rotate every vertex into the viewing pose.
        let (sx, cx) = ROT_X.sin_cos();
        let (sy, cy) = ROT_Y.sin_cos();
        let rotated: Vec<[f32; 3]> = mesh
            .vertices
            .iter()
            .map(|v| {
                // Rotate around Y, then around X.
                let x1 = v[0] * cy + v[2] * sy;
                let z1 = -v[0] * sy + v[2] * cy;
                let y2 = v[1] * cx - z1 * sx;
                let z2 = v[1] * sx + z1 * cx;
                [x1, y2, z2]
            })
            .collect();

        // Bounding box of the rotated XY footprint, to fit the image.
        let (mut min_x, mut max_x) = (f32::INFINITY, f32::NEG_INFINITY);
        let (mut min_y, mut max_y) = (f32::INFINITY, f32::NEG_INFINITY);
        for p in &rotated {
            min_x = min_x.min(p[0]);
            max_x = max_x.max(p[0]);
            min_y = min_y.min(p[1]);
            max_y = max_y.max(p[1]);
        }

        let span = (max_x - min_x).max(max_y - min_y).max(f32::EPSILON);
        let draw = res as f32 * (1.0 - 2.0 * MARGIN);
        let scale = draw / span;
        // Centre the footprint within the image.
        let off_x = (res as f32 - (max_x - min_x) * scale) * 0.5;
        let off_y = (res as f32 - (max_y - min_y) * scale) * 0.5;

        let project = |p: [f32; 3]| -> Projected {
            let sx = (p[0] - min_x) * scale + off_x;
            // Flip Y so positive points upward in the image.
            let sy = res as f32 - ((p[1] - min_y) * scale + off_y);
            Projected {
                x: sx,
                y: sy,
                depth: p[2],
                world: p,
            }
        };

        // Light pointing toward the viewer and slightly down-right.
        let light = {
            let l = [-0.4f32, -0.5, 1.0];
            let len = (l[0] * l[0] + l[1] * l[1] + l[2] * l[2]).sqrt();
            [l[0] / len, l[1] / len, l[2] / len]
        };

        let mut depth_buffer = vec![f32::NEG_INFINITY; width * height];

        for face in &mesh.faces {
            let v0 = project(rotated[face[0] as usize]);
            let v1 = project(rotated[face[1] as usize]);
            let v2 = project(rotated[face[2] as usize]);

            // Face normal from the rotated world positions.
            let a = v0.world;
            let b = v1.world;
            let c = v2.world;
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let normal = [
                ab[1] * ac[2] - ab[2] * ac[1],
                ab[2] * ac[0] - ab[0] * ac[2],
                ab[0] * ac[1] - ab[1] * ac[0],
            ];
            let nlen = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2])
                .sqrt()
                .max(f32::EPSILON);
            // Two-sided shading: light either face of the surface.
            let diffuse =
                ((normal[0] * light[0] + normal[1] * light[1] + normal[2] * light[2]) / nlen).abs();
            let shade = (AMBIENT + (1.0 - AMBIENT) * diffuse).clamp(0.0, 1.0);

            let p0 = (v0.x, v0.y);
            let p1 = (v1.x, v1.y);
            let p2 = (v2.x, v2.y);

            let area = edge(p0, p1, p2);
            if area.abs() < f32::EPSILON {
                continue; // Degenerate triangle.
            }

            let min_px = p0.0.min(p1.0).min(p2.0).floor().max(0.0) as usize;
            let max_px =
                (p0.0.max(p1.0).max(p2.0).ceil() as i64).clamp(0, width as i64 - 1) as usize;
            let min_py = p0.1.min(p1.1).min(p2.1).floor().max(0.0) as usize;
            let max_py =
                (p0.1.max(p1.1).max(p2.1).ceil() as i64).clamp(0, height as i64 - 1) as usize;

            for py in min_py..=max_py {
                for px in min_px..=max_px {
                    let p = (px as f32 + 0.5, py as f32 + 0.5);
                    let w0 = edge(p1, p2, p);
                    let w1 = edge(p2, p0, p);
                    let w2 = edge(p0, p1, p);

                    let inside = (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0)
                        || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0);
                    if !inside {
                        continue;
                    }

                    let sum = w0 + w1 + w2;
                    if sum.abs() < f32::EPSILON {
                        continue;
                    }
                    let depth = (w0 * v0.depth + w1 * v1.depth + w2 * v2.depth) / sum;

                    let idx = py * width + px;
                    if depth > depth_buffer[idx] {
                        depth_buffer[idx] = depth;
                        let o = idx * 3;
                        pixels[o] = (base[0] * shade).clamp(0.0, 255.0) as u8;
                        pixels[o + 1] = (base[1] * shade).clamp(0.0, 255.0) as u8;
                        pixels[o + 2] = (base[2] * shade).clamp(0.0, 255.0) as u8;
                    }
                }
            }
        }

        self.output = Some(Arc::new(RawImage {
            width: res,
            height: res,
            pixels,
        }));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> Arc<Mesh3D> {
        Arc::new(Mesh3D::new(
            vec![[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        ))
    }

    #[test]
    fn test_render_produces_rgb_image() {
        let mut proc = ModelRenderProcessor::new("render".into());
        proc.set_input(vec![triangle(), Arc::new(64u32)]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let img = output[0].downcast_ref::<RawImage>().unwrap();
        assert_eq!(img.width, 64);
        assert_eq!(img.height, 64);
        assert_eq!(img.pixels.len(), 64 * 64 * 3);
    }

    #[test]
    fn test_render_draws_something() {
        let mut proc = ModelRenderProcessor::new("render".into());
        proc.set_input(vec![triangle(), Arc::new(64u32)]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let img = output[0].downcast_ref::<RawImage>().unwrap();
        // At least some pixels must differ from the background colour.
        let bg = [30u8, 30u8, 40u8];
        let drawn = img
            .pixels
            .chunks(3)
            .any(|p| p[0] != bg[0] || p[1] != bg[1] || p[2] != bg[2]);
        assert!(drawn, "expected the triangle to colour some pixels");
    }

    #[test]
    fn test_render_defaults_resolution_without_second_input() {
        let mut proc = ModelRenderProcessor::new("render".into());
        proc.set_input(vec![triangle()]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let img = output[0].downcast_ref::<RawImage>().unwrap();
        assert_eq!(img.width, DEFAULT_RESOLUTION);
        assert_eq!(img.height, DEFAULT_RESOLUTION);
    }

    #[test]
    fn test_render_resolution_from_string() {
        let mut proc = ModelRenderProcessor::new("render".into());
        proc.set_input(vec![triangle(), Arc::new("32".to_string())])
            .unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let img = output[0].downcast_ref::<RawImage>().unwrap();
        assert_eq!(img.width, 32);
    }

    #[test]
    fn test_render_empty_mesh_returns_background() {
        let mut proc = ModelRenderProcessor::new("render".into());
        let empty = Arc::new(Mesh3D::new(vec![], vec![]));
        proc.set_input(vec![empty, Arc::new(8u32)]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let img = output[0].downcast_ref::<RawImage>().unwrap();
        assert_eq!(img.pixels.len(), 8 * 8 * 3);
    }

    #[test]
    fn test_render_missing_input_fails() {
        let mut proc = ModelRenderProcessor::new("render".into());
        assert!(matches!(
            proc.process().unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_render_wrong_mesh_type_fails() {
        let mut proc = ModelRenderProcessor::new("render".into());
        let bad: Arc<dyn std::any::Any + Send + Sync> = Arc::new(42u32);
        assert!(matches!(
            proc.set_input(vec![bad]).unwrap_err(),
            ProcessorError::InvalidInput(_)
        ));
    }
}
