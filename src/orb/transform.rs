use nalgebra::Matrix4;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

/// A 4x4 affine transform stored as 128 bytes (16 x f64, column-major, little-endian).
/// Wraps nalgebra::Matrix4<f64> with Orb BLOB serialization.
#[derive(Debug, Clone, Copy)]
pub struct Transform(pub Matrix4<f64>);

const TRANSFORM_BLOB_SIZE: usize = 128;

impl Transform {
    pub fn identity() -> Self {
        Self(Matrix4::identity())
    }

    pub fn from_translation(x: f64, y: f64, z: f64) -> Self {
        Self(Matrix4::new_translation(&nalgebra::Vector3::new(x, y, z)))
    }

    pub fn to_blob(&self) -> [u8; TRANSFORM_BLOB_SIZE] {
        let mut buf = [0u8; TRANSFORM_BLOB_SIZE];
        // nalgebra stores column-major, which matches the spec layout
        for (i, val) in self.0.iter().enumerate() {
            let bytes = val.to_le_bytes();
            buf[i * 8..i * 8 + 8].copy_from_slice(&bytes);
        }
        buf
    }

    pub fn from_blob(bytes: &[u8]) -> Result<Self, TransformError> {
        if bytes.len() != TRANSFORM_BLOB_SIZE {
            return Err(TransformError::InvalidSize(bytes.len()));
        }
        let mut vals = [0.0f64; 16];
        for (i, val) in vals.iter_mut().enumerate() {
            let start = i * 8;
            let arr: [u8; 8] = bytes[start..start + 8]
                .try_into()
                .expect("slice length is checked");
            *val = f64::from_le_bytes(arr);
        }
        // nalgebra::Matrix4 takes column-major data via from_column_slice
        Ok(Self(Matrix4::from_column_slice(&vals)))
    }

    pub fn as_matrix(&self) -> &Matrix4<f64> {
        &self.0
    }

    /// Convert to f32 matrix for GPU uniforms.
    pub fn to_f32(&self) -> Matrix4<f32> {
        self.0.cast::<f32>()
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl ToSql for Transform {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(rusqlite::types::Value::Blob(
            self.to_blob().to_vec(),
        )))
    }
}

impl FromSql for Transform {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Blob(bytes) => Self::from_blob(bytes).map_err(|e| FromSqlError::Other(Box::new(e))),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TransformError {
    #[error("transform BLOB must be exactly 128 bytes, got {0}")]
    InvalidSize(usize),
}
