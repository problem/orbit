use anyhow::{bail, Result};

/// Display-ready mesh data corresponding to `orb_geometry_mesh`.
#[derive(Debug, Clone)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub edges: Option<Vec<[u32; 2]>>,
}

impl MeshData {
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    // --- BLOB packing (structure-of-arrays, little-endian, per spec §6.3) ---

    pub fn positions_to_blob(&self) -> Vec<u8> {
        let bytes: &[u8] = bytemuck::cast_slice(&self.positions);
        bytes.to_vec()
    }

    pub fn positions_from_blob(blob: &[u8]) -> Result<Vec<[f32; 3]>> {
        if blob.len() % 12 != 0 {
            bail!(
                "positions BLOB length {} is not divisible by 12",
                blob.len()
            );
        }
        Ok(bytemuck::cast_slice::<u8, [f32; 3]>(blob).to_vec())
    }

    pub fn normals_to_blob(&self) -> Vec<u8> {
        let bytes: &[u8] = bytemuck::cast_slice(&self.normals);
        bytes.to_vec()
    }

    pub fn normals_from_blob(blob: &[u8]) -> Result<Vec<[f32; 3]>> {
        if blob.len() % 12 != 0 {
            bail!("normals BLOB length {} is not divisible by 12", blob.len());
        }
        Ok(bytemuck::cast_slice::<u8, [f32; 3]>(blob).to_vec())
    }

    pub fn indices_to_blob(&self) -> Vec<u8> {
        let bytes: &[u8] = bytemuck::cast_slice(&self.indices);
        bytes.to_vec()
    }

    pub fn indices_from_blob(blob: &[u8]) -> Result<Vec<u32>> {
        if blob.len() % 4 != 0 {
            bail!("indices BLOB length {} is not divisible by 4", blob.len());
        }
        Ok(bytemuck::cast_slice::<u8, u32>(blob).to_vec())
    }

    /// Generate a unit cube centered at the origin with correct normals.
    pub fn cube(size: f32) -> Self {
        let h = size / 2.0;
        // 24 vertices (4 per face, 6 faces) for correct per-face normals
        #[rustfmt::skip]
        let positions: Vec<[f32; 3]> = vec![
            // Front face (z+)
            [-h, -h,  h], [ h, -h,  h], [ h,  h,  h], [-h,  h,  h],
            // Back face (z-)
            [ h, -h, -h], [-h, -h, -h], [-h,  h, -h], [ h,  h, -h],
            // Top face (y+)
            [-h,  h,  h], [ h,  h,  h], [ h,  h, -h], [-h,  h, -h],
            // Bottom face (y-)
            [-h, -h, -h], [ h, -h, -h], [ h, -h,  h], [-h, -h,  h],
            // Right face (x+)
            [ h, -h,  h], [ h, -h, -h], [ h,  h, -h], [ h,  h,  h],
            // Left face (x-)
            [-h, -h, -h], [-h, -h,  h], [-h,  h,  h], [-h,  h, -h],
        ];
        #[rustfmt::skip]
        let normals: Vec<[f32; 3]> = vec![
            [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],
            [0.0, 0.0,-1.0], [0.0, 0.0,-1.0], [0.0, 0.0,-1.0], [0.0, 0.0,-1.0],
            [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0],
            [0.0,-1.0, 0.0], [0.0,-1.0, 0.0], [0.0,-1.0, 0.0], [0.0,-1.0, 0.0],
            [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0],
            [-1.0,0.0, 0.0], [-1.0,0.0, 0.0], [-1.0,0.0, 0.0], [-1.0,0.0, 0.0],
        ];
        let mut indices = Vec::new();
        for face in 0..6u32 {
            let base = face * 4;
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        Self {
            positions,
            normals,
            indices,
            edges: None,
        }
    }
}
