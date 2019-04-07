use core::borrow::{Borrow, BorrowMut};
use core::cmp;
use core::fmt::{self, Debug};
use core::ops::{Deref, DerefMut};

#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use serde::de::{Deserialize, Deserializer, Error, SeqAccess, Visitor};
use serde::ser::{Serialize, Serializer};

use crate::Bytes;

/// Wrapper around `Vec<u8>` to serialize and deserialize efficiently.
///
/// ```
/// use std::collections::HashMap;
/// use std::io;
///
/// use serde_bytes::ByteBuf;
///
/// fn deserialize_bytebufs() -> bincode::Result<()> {
///     let example_data = [
///         2, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 116,
///         119, 111, 1, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 111, 110, 101];
///
///     let map: HashMap<u32, ByteBuf> = bincode::deserialize(&example_data[..])?;
///
///     println!("{:?}", map);
///
///     Ok(())
/// }
/// #
/// # fn main() {
/// #     deserialize_bytebufs().unwrap();
/// # }
/// ```
#[derive(Clone, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ByteBuf {
    bytes: Vec<u8>,
}

impl ByteBuf {
    /// Construct a new, empty `ByteBuf`.
    pub fn new() -> Self {
        ByteBuf::from(Vec::new())
    }

    /// Construct a new, empty `ByteBuf` with the specified capacity.
    pub fn with_capacity(cap: usize) -> Self {
        ByteBuf::from(Vec::with_capacity(cap))
    }

    /// Wrap existing bytes in a `ByteBuf`.
    pub fn from<T: Into<Vec<u8>>>(bytes: T) -> Self {
        ByteBuf {
            bytes: bytes.into(),
        }
    }

    /// Unwrap the vector of byte underlying this `ByteBuf`.
    pub fn into_inner(self) -> Vec<u8> {
        self.bytes
    }
}

impl Debug for ByteBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.bytes, f)
    }
}

impl From<Vec<u8>> for ByteBuf {
    fn from(bytes: Vec<u8>) -> Self {
        ByteBuf::from(bytes)
    }
}

impl AsRef<[u8]> for ByteBuf {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl AsMut<[u8]> for ByteBuf {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
}

impl Deref for ByteBuf {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl DerefMut for ByteBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bytes
    }
}

impl Borrow<Bytes> for ByteBuf {
    fn borrow(&self) -> &Bytes {
        Bytes::new(&self.bytes)
    }
}

impl BorrowMut<Bytes> for ByteBuf {
    fn borrow_mut(&mut self) -> &mut Bytes {
        unsafe { &mut *(&mut self.bytes as &mut [u8] as *mut [u8] as *mut Bytes) }
    }
}

impl Serialize for ByteBuf {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.bytes)
    }
}

struct ByteBufVisitor;

impl<'de> Visitor<'de> for ByteBufVisitor {
    type Value = ByteBuf;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("byte array")
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<ByteBuf, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let len = cmp::min(visitor.size_hint().unwrap_or(0), 4096);
        let mut values = Vec::with_capacity(len);

        while let Some(value) = visitor.next_element()? {
            values.push(value);
        }

        Ok(ByteBuf::from(values))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<ByteBuf, E>
    where
        E: Error,
    {
        Ok(ByteBuf::from(v))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<ByteBuf, E>
    where
        E: Error,
    {
        Ok(ByteBuf::from(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<ByteBuf, E>
    where
        E: Error,
    {
        Ok(ByteBuf::from(v))
    }

    fn visit_string<E>(self, v: String) -> Result<ByteBuf, E>
    where
        E: Error,
    {
        Ok(ByteBuf::from(v))
    }
}

impl<'de> Deserialize<'de> for ByteBuf {
    fn deserialize<D>(deserializer: D) -> Result<ByteBuf, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_byte_buf(ByteBufVisitor)
    }
}
