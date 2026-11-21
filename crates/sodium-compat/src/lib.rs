//! Clean-room chunk mesh data structures for a Rust voxel renderer.
//! This crate is renderer-agnostic; a game adapter supplies block geometry and GPU upload.

pub mod compact;
pub use compact::{EncodeError, InputVertex, encode_quad};

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [u8; 4],
    pub uv: [f32; 2],
    pub light: [u8; 2],
    pub normal: [i8; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderLayer {
    Solid,
    Cutout,
    CutoutMipped,
    Translucent,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    /// Triangle indices into vertices.
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn push_quad(&mut self, vertices: [Vertex; 4]) -> Result<(), MeshError> {
        let base = u32::try_from(self.vertices.len()).map_err(|_| MeshError::TooManyVertices)?;
        base.checked_add(3).ok_or(MeshError::TooManyVertices)?;
        self.vertices.extend(vertices);
        self.indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
        Ok(())
    }

    pub fn is_empty(&self) -> bool { self.indices.is_empty() }

    pub fn validate(&self) -> Result<(), MeshError> {
        if self.indices.len() % 3 != 0 { return Err(MeshError::NonTriangularIndexCount); }
        let count = self.vertices.len() as u64;
        if self.indices.iter().any(|&i| u64::from(i) >= count) {
            return Err(MeshError::IndexOutOfBounds);
        }
        if self.vertices.iter().any(|v| v.position.iter().chain(v.uv.iter()).any(|x| !x.is_finite())) {
            return Err(MeshError::NonFiniteVertex);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChunkMesh {
    layers: BTreeMap<RenderLayer, Mesh>,
}

impl ChunkMesh {
    pub fn layer(&self, layer: RenderLayer) -> Option<&Mesh> { self.layers.get(&layer) }

    pub fn layer_mut(&mut self, layer: RenderLayer) -> &mut Mesh {
        self.layers.entry(layer).or_default()
    }

    pub fn layers(&self) -> impl Iterator<Item = (RenderLayer, &Mesh)> {
        self.layers.iter().map(|(layer, mesh)| (*layer, mesh))
    }

    pub fn validate(&self) -> Result<(), MeshError> {
        for mesh in self.layers.values() { mesh.validate()?; }
        Ok(())
    }

    pub fn total_vertices(&self) -> usize {
        self.layers.values().map(|mesh| mesh.vertices.len()).sum()
    }

    pub fn total_indices(&self) -> usize {
        self.layers.values().map(|mesh| mesh.indices.len()).sum()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshError {
    TooManyVertices,
    NonTriangularIndexCount,
    IndexOutOfBounds,
    NonFiniteVertex,
}

impl std::fmt::Display for MeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for MeshError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn vertex(x: f32, y: f32) -> Vertex {
        Vertex { position: [x, y, 0.0], ..Vertex::default() }
    }

    #[test]
    fn quad_generates_two_triangles_with_local_indices() {
        let mut mesh = Mesh::default();
        mesh.push_quad([vertex(0.0,0.0), vertex(1.0,0.0), vertex(1.0,1.0), vertex(0.0,1.0)]).unwrap();
        assert_eq!(mesh.indices, [0,1,2,2,3,0]);
        mesh.push_quad([vertex(2.0,0.0), vertex(3.0,0.0), vertex(3.0,1.0), vertex(2.0,1.0)]).unwrap();
        assert_eq!(&mesh.indices[6..], [4,5,6,6,7,4]);
        mesh.validate().unwrap();
    }

    #[test]
    fn chunk_keeps_render_layers_separate() {
        let mut chunk = ChunkMesh::default();
        chunk.layer_mut(RenderLayer::Solid).push_quad([vertex(0.0,0.0),vertex(1.0,0.0),vertex(1.0,1.0),vertex(0.0,1.0)]).unwrap();
        chunk.layer_mut(RenderLayer::Translucent).push_quad([vertex(0.0,0.0),vertex(1.0,0.0),vertex(1.0,1.0),vertex(0.0,1.0)]).unwrap();
        assert_eq!(chunk.layers().count(), 2);
        assert_eq!(chunk.total_vertices(), 8);
        assert_eq!(chunk.total_indices(), 12);
    }

    #[test]
    fn validation_rejects_nan_positions() {
        let mut mesh = Mesh::default();
        mesh.vertices.push(vertex(f32::NAN, 0.0));
        assert_eq!(mesh.validate(), Err(MeshError::NonFiniteVertex));
    }
}
