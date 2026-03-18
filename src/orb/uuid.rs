use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use std::fmt;

/// A UUIDv7 identifier for Orb entities, materials, textures, etc.
/// Stored as a 16-byte BLOB in SQLite per the Orb format spec.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrbId(uuid::Uuid);

impl OrbId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }

    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(uuid::Uuid::from_bytes(bytes))
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }
}

impl Default for OrbId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for OrbId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OrbId({})", self.0)
    }
}

impl fmt::Display for OrbId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ToSql for OrbId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Borrowed(ValueRef::Blob(self.0.as_bytes())))
    }
}

impl FromSql for OrbId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Blob(bytes) => {
                let arr: [u8; 16] = bytes
                    .try_into()
                    .map_err(|_| FromSqlError::InvalidBlobSize { expected_size: 16, blob_size: bytes.len() })?;
                Ok(Self::from_bytes(arr))
            }
            _ => Err(FromSqlError::InvalidType),
        }
    }
}
