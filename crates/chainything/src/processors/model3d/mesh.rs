use crate::processors::base_processor::ProcessorError;
use std::fmt::Write as _;

/// A triangle mesh loaded from — or destined for — a 3D model file.
///
/// Geometry is stored as a flat list of vertex positions plus a list of triangle
/// faces that index into it. This is intentionally minimal: enough to load,
/// transform, and re-export common formats such as Wavefront OBJ, without
/// pulling in a heavy 3D dependency.
#[derive(Clone, Debug, PartialEq)]
pub struct Mesh3D {
    /// Vertex positions in model space, one `[x, y, z]` per vertex.
    pub vertices: Vec<[f32; 3]>,
    /// Triangles, each holding three 0-based indices into
    /// [`vertices`](Self::vertices).
    pub faces: Vec<[u32; 3]>,
}

impl Mesh3D {
    pub fn new(vertices: Vec<[f32; 3]>, faces: Vec<[u32; 3]>) -> Self {
        Self { vertices, faces }
    }

    /// Number of vertices in the mesh.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Number of triangular faces in the mesh.
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// Parses a Wavefront OBJ document into a [`Mesh3D`].
    ///
    /// Only vertex (`v`) and face (`f`) records are honoured; normals, texture
    /// coordinates, materials and groups are ignored. Faces with more than three
    /// vertices are triangulated with a simple fan. Face indices may be written
    /// as `v`, `v/vt`, `v/vt/vn` or `v//vn`; only the vertex component is read.
    /// Negative (relative) indices are supported.
    ///
    /// - **Errors:** [`ProcessorError::ComputingError`] if a coordinate or index
    ///   cannot be parsed, or if a face references an out-of-range vertex.
    pub fn from_obj(content: &str) -> Result<Self, ProcessorError> {
        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut faces: Vec<[u32; 3]> = Vec::new();

        for (line_no, raw_line) in content.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut tokens = line.split_whitespace();
            let Some(keyword) = tokens.next() else {
                continue;
            };

            match keyword {
                "v" => {
                    let coords: Vec<f32> = tokens
                        .take(3)
                        .map(|t| {
                            t.parse::<f32>().map_err(|_| {
                                ProcessorError::ComputingError(format!(
                                    "Invalid vertex coordinate '{}' on line {}",
                                    t,
                                    line_no + 1
                                ))
                            })
                        })
                        .collect::<Result<_, _>>()?;

                    if coords.len() < 3 {
                        return Err(ProcessorError::ComputingError(format!(
                            "Vertex on line {} needs 3 coordinates, got {}",
                            line_no + 1,
                            coords.len()
                        )));
                    }

                    vertices.push([coords[0], coords[1], coords[2]]);
                }
                "f" => {
                    let mut indices: Vec<u32> = Vec::new();
                    for token in tokens {
                        // Keep only the vertex component of `v`, `v/vt`,
                        // `v/vt/vn` or `v//vn`.
                        let vertex_part = token.split('/').next().unwrap_or(token);
                        let raw: i64 = vertex_part.parse().map_err(|_| {
                            ProcessorError::ComputingError(format!(
                                "Invalid face index '{}' on line {}",
                                vertex_part,
                                line_no + 1
                            ))
                        })?;

                        // OBJ indices are 1-based; negatives count back from the
                        // current end of the vertex list.
                        let resolved = if raw < 0 {
                            vertices.len() as i64 + raw
                        } else {
                            raw - 1
                        };

                        if resolved < 0 || resolved as usize >= vertices.len() {
                            return Err(ProcessorError::ComputingError(format!(
                                "Face index {} out of range on line {}",
                                raw,
                                line_no + 1
                            )));
                        }

                        indices.push(resolved as u32);
                    }

                    if indices.len() < 3 {
                        return Err(ProcessorError::ComputingError(format!(
                            "Face on line {} needs at least 3 vertices, got {}",
                            line_no + 1,
                            indices.len()
                        )));
                    }

                    // Fan-triangulate any polygon into triangles.
                    for i in 1..indices.len() - 1 {
                        faces.push([indices[0], indices[i], indices[i + 1]]);
                    }
                }
                _ => {}
            }
        }

        Ok(Mesh3D::new(vertices, faces))
    }

    /// Serializes this mesh as a Wavefront OBJ document (vertices + faces).
    ///
    /// Face indices are written 1-based, as the OBJ format requires.
    pub fn to_obj(&self) -> String {
        let mut out = String::new();
        out.push_str("# Generated by chainything\n");

        for v in &self.vertices {
            // `write!` into a String is infallible.
            let _ = writeln!(out, "v {} {} {}", v[0], v[1], v[2]);
        }

        for f in &self.faces {
            let _ = writeln!(out, "f {} {} {}", f[0] + 1, f[1] + 1, f[2] + 1);
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CUBE_CORNER: &str = "\
# a tiny triangle
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
f 1 2 3
";

    #[test]
    fn test_from_obj_parses_vertices_and_faces() {
        let mesh = Mesh3D::from_obj(CUBE_CORNER).unwrap();
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.face_count(), 1);
        assert_eq!(mesh.vertices[1], [1.0, 0.0, 0.0]);
        assert_eq!(mesh.faces[0], [0, 1, 2]);
    }

    #[test]
    fn test_from_obj_ignores_texture_and_normal_components() {
        let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1/1/1 2/2/2 3/3/3\n";
        let mesh = Mesh3D::from_obj(obj).unwrap();
        assert_eq!(mesh.faces[0], [0, 1, 2]);
    }

    #[test]
    fn test_from_obj_triangulates_quads() {
        let obj = "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n";
        let mesh = Mesh3D::from_obj(obj).unwrap();
        // A quad fans into two triangles.
        assert_eq!(mesh.face_count(), 2);
        assert_eq!(mesh.faces[0], [0, 1, 2]);
        assert_eq!(mesh.faces[1], [0, 2, 3]);
    }

    #[test]
    fn test_from_obj_supports_negative_indices() {
        let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf -3 -2 -1\n";
        let mesh = Mesh3D::from_obj(obj).unwrap();
        assert_eq!(mesh.faces[0], [0, 1, 2]);
    }

    #[test]
    fn test_from_obj_invalid_coordinate_fails() {
        let obj = "v 0 nope 0\n";
        assert!(matches!(
            Mesh3D::from_obj(obj).unwrap_err(),
            ProcessorError::ComputingError(_)
        ));
    }

    #[test]
    fn test_from_obj_out_of_range_face_fails() {
        let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 9\n";
        assert!(matches!(
            Mesh3D::from_obj(obj).unwrap_err(),
            ProcessorError::ComputingError(_)
        ));
    }

    #[test]
    fn test_to_obj_round_trips() {
        let mesh = Mesh3D::from_obj(CUBE_CORNER).unwrap();
        let serialized = mesh.to_obj();
        let reparsed = Mesh3D::from_obj(&serialized).unwrap();
        assert_eq!(mesh, reparsed);
    }
}
