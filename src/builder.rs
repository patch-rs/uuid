// Copyright 2013-2014 The Rust Project Developers.
// Copyright 2018 The Uuid Project Developers.
//
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A Builder type for [`Uuid`]s.
//!
//! [`Uuid`]: ../struct.Uuid.html

use core::convert::TryInto;
use core::time::Duration;
use crate::{error::*, Bytes, Uuid, Variant, Version};

/// A builder struct for creating a UUID.
///
/// This type is useful if you need to mutate individual fields of a [`Uuid`]
/// while constructing it. Since the [`Uuid`] type is `Copy`, it doesn't offer
/// any methods to mutate in place. They live on the `Builder` instead.
///
/// # Examples
///
/// Creating a v4 UUID from externally generated bytes:
///
/// ```
/// # use uuid::{Builder, Version, Variant};
/// # let rng = || [
/// #     70, 235, 208, 238, 14, 109, 67, 201, 185, 13, 204, 195, 90,
/// # 145, 63, 62,
/// # ];
/// let random_bytes = rng();
///
/// let uuid = Builder::from_random_bytes(random_bytes).into_uuid();
///
/// assert_eq!(Some(Version::Random), uuid.get_version());
/// assert_eq!(Variant::RFC4122, uuid.get_variant());
/// ```
#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct Builder(Uuid);

impl Uuid {
    /// The 'nil UUID'.
    ///
    /// The nil UUID is a special form of UUID that is specified to have all
    /// 128 bits set to zero, as defined in [IETF RFC 4122 Section 4.1.7][RFC].
    ///
    /// [RFC]: https://tools.ietf.org/html/rfc4122.html#section-4.1.7
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::Uuid;
    /// let uuid = Uuid::nil();
    ///
    /// assert_eq!(
    ///     "00000000-0000-0000-0000-000000000000",
    ///     uuid.hyphenated().to_string(),
    /// );
    /// ```
    pub const fn nil() -> Self {
        Uuid::from_bytes([0; 16])
    }

    /// The 'max UUID'.
    ///
    /// The max UUID is a special form of UUID that is specified to have all
    /// 128 bits set to one, as defined in [IETF RFC 4122 Update Section 5.4][Draft RFC].
    ///
    /// [Draft RFC]: https://datatracker.ietf.org/doc/html/draft-peabody-dispatch-new-uuid-format-04#page-12
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::Uuid;
    /// let uuid = Uuid::max();
    ///
    /// assert_eq!(
    ///     "ffffffff-ffff-ffff-ffff-ffffffffffff",
    ///     uuid.hyphenated().to_string(),
    /// );
    /// ```
    pub const fn max() -> Self {
        Uuid::from_bytes([0xFF; 16])
    }

    /// Creates a UUID from four field values.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::Uuid;
    /// let d1 = 0xa1a2a3a4;
    /// let d2 = 0xb1b2;
    /// let d3 = 0xc1c2;
    /// let d4 = [0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8];
    ///
    /// let uuid = Uuid::from_fields(d1, d2, d3, &d4);
    ///
    /// assert_eq!(
    ///     "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8",
    ///     uuid.hyphenated().to_string(),
    /// );
    /// ```
    pub const fn from_fields(d1: u32, d2: u16, d3: u16, d4: &[u8; 8]) -> Uuid {
        Uuid::from_bytes([
            (d1 >> 24) as u8,
            (d1 >> 16) as u8,
            (d1 >> 8) as u8,
            d1 as u8,
            (d2 >> 8) as u8,
            d2 as u8,
            (d3 >> 8) as u8,
            d3 as u8,
            d4[0],
            d4[1],
            d4[2],
            d4[3],
            d4[4],
            d4[5],
            d4[6],
            d4[7],
        ])
    }

    /// Creates a UUID from four field values in little-endian order.
    ///
    /// The bytes in the `d1`, `d2` and `d3` fields will be flipped to convert
    /// into big-endian order. This is based on the endianness of the UUID,
    /// rather than the target environment so bytes will be flipped on both
    /// big and little endian machines.
    ///
    /// # Examples
    ///
    /// ```
    /// # use uuid::Uuid;
    /// let d1 = 0xa1a2a3a4;
    /// let d2 = 0xb1b2;
    /// let d3 = 0xc1c2;
    /// let d4 = [0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8];
    ///
    /// let uuid = Uuid::from_fields_le(d1, d2, d3, &d4);
    ///
    /// assert_eq!(
    ///     "a4a3a2a1-b2b1-c2c1-d1d2-d3d4d5d6d7d8",
    ///     uuid.hyphenated().to_string(),
    /// );
    /// ```
    pub const fn from_fields_le(d1: u32, d2: u16, d3: u16, d4: &[u8; 8]) -> Uuid {
        Uuid::from_bytes([
            d1 as u8,
            (d1 >> 8) as u8,
            (d1 >> 16) as u8,
            (d1 >> 24) as u8,
            (d2) as u8,
            (d2 >> 8) as u8,
            d3 as u8,
            (d3 >> 8) as u8,
            d4[0],
            d4[1],
            d4[2],
            d4[3],
            d4[4],
            d4[5],
            d4[6],
            d4[7],
        ])
    }

    /// Creates a UUID from a 128bit value.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::Uuid;
    /// let v = 0xa1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8u128;
    ///
    /// let uuid = Uuid::from_u128(v);
    ///
    /// assert_eq!(
    ///     "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8",
    ///     uuid.hyphenated().to_string(),
    /// );
    /// ```
    pub const fn from_u128(v: u128) -> Self {
        Uuid::from_bytes([
            (v >> 120) as u8,
            (v >> 112) as u8,
            (v >> 104) as u8,
            (v >> 96) as u8,
            (v >> 88) as u8,
            (v >> 80) as u8,
            (v >> 72) as u8,
            (v >> 64) as u8,
            (v >> 56) as u8,
            (v >> 48) as u8,
            (v >> 40) as u8,
            (v >> 32) as u8,
            (v >> 24) as u8,
            (v >> 16) as u8,
            (v >> 8) as u8,
            v as u8,
        ])
    }

    /// Creates a UUID from a 128bit value in little-endian order.
    ///
    /// The entire value will be flipped to convert into big-endian order.
    /// This is based on the endianness of the UUID, rather than the target
    /// environment so bytes will be flipped on both big and little endian
    /// machines.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::Uuid;
    /// let v = 0xa1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8u128;
    ///
    /// let uuid = Uuid::from_u128_le(v);
    ///
    /// assert_eq!(
    ///     "d8d7d6d5-d4d3-d2d1-c2c1-b2b1a4a3a2a1",
    ///     uuid.hyphenated().to_string(),
    /// );
    /// ```
    pub const fn from_u128_le(v: u128) -> Self {
        Uuid::from_bytes([
            v as u8,
            (v >> 8) as u8,
            (v >> 16) as u8,
            (v >> 24) as u8,
            (v >> 32) as u8,
            (v >> 40) as u8,
            (v >> 48) as u8,
            (v >> 56) as u8,
            (v >> 64) as u8,
            (v >> 72) as u8,
            (v >> 80) as u8,
            (v >> 88) as u8,
            (v >> 96) as u8,
            (v >> 104) as u8,
            (v >> 112) as u8,
            (v >> 120) as u8,
        ])
    }

    /// Creates a UUID from two 64bit values.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::Uuid;
    /// let hi = 0xa1a2a3a4b1b2c1c2u64;
    /// let lo = 0xd1d2d3d4d5d6d7d8u64;
    ///
    /// let uuid = Uuid::from_u64_pair(hi, lo);
    ///
    /// assert_eq!(
    ///     "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8",
    ///     uuid.hyphenated().to_string(),
    /// );
    /// ```
    pub const fn from_u64_pair(high_bits: u64, low_bits: u64) -> Self {
        Uuid::from_bytes([
            (high_bits >> 56) as u8,
            (high_bits >> 48) as u8,
            (high_bits >> 40) as u8,
            (high_bits >> 32) as u8,
            (high_bits >> 24) as u8,
            (high_bits >> 16) as u8,
            (high_bits >> 8) as u8,
            high_bits as u8,
            (low_bits >> 56) as u8,
            (low_bits >> 48) as u8,
            (low_bits >> 40) as u8,
            (low_bits >> 32) as u8,
            (low_bits >> 24) as u8,
            (low_bits >> 16) as u8,
            (low_bits >> 8) as u8,
            low_bits as u8,
        ])
    }

    /// Creates a UUID using the supplied bytes.
    ///
    /// # Errors
    ///
    /// This function will return an error if `b` has any length other than 16.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # fn main() -> Result<(), uuid::Error> {
    /// # use uuid::Uuid;
    /// let bytes = [
    ///     0xa1, 0xa2, 0xa3, 0xa4,
    ///     0xb1, 0xb2,
    ///     0xc1, 0xc2,
    ///     0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
    /// ];
    ///
    /// let uuid = Uuid::from_slice(&bytes)?;
    ///
    /// assert_eq!(
    ///     "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8",
    ///     uuid.hyphenated().to_string(),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_slice(b: &[u8]) -> Result<Uuid, Error> {
        if b.len() != 16 {
            return Err(Error(ErrorKind::ByteLength { len: b.len() }));
        }

        let mut bytes: Bytes = [0; 16];
        bytes.copy_from_slice(b);
        Ok(Uuid::from_bytes(bytes))
    }

    /// Creates a UUID using the supplied bytes in little endian order.
    ///
    /// The individual fields encoded in the buffer will be flipped.
    ///
    /// # Errors
    ///
    /// This function will return an error if `b` has any length other than 16.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # fn main() -> Result<(), uuid::Error> {
    /// # use uuid::Uuid;
    /// let bytes = [
    ///     0xa1, 0xa2, 0xa3, 0xa4,
    ///     0xb1, 0xb2,
    ///     0xc1, 0xc2,
    ///     0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
    /// ];
    ///
    /// let uuid = Uuid::from_slice_le(&bytes)?;
    ///
    /// assert_eq!(
    ///     uuid.hyphenated().to_string(),
    ///     "a4a3a2a1-b2b1-c2c1-d1d2-d3d4d5d6d7d8"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_slice_le(b: &[u8]) -> Result<Uuid, Error> {
        if b.len() != 16 {
            return Err(Error(ErrorKind::ByteLength { len: b.len() }));
        }

        let mut bytes: Bytes = [0; 16];
        bytes.copy_from_slice(b);
        Ok(Uuid::from_bytes_le(bytes))
    }

    /// Creates a UUID using the supplied bytes.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # fn main() -> Result<(), uuid::Error> {
    /// # use uuid::Uuid;
    /// let bytes = [
    ///     0xa1, 0xa2, 0xa3, 0xa4,
    ///     0xb1, 0xb2,
    ///     0xc1, 0xc2,
    ///     0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
    /// ];
    ///
    /// let uuid = Uuid::from_bytes(bytes);
    ///
    /// assert_eq!(
    ///     uuid.hyphenated().to_string(),
    ///     "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub const fn from_bytes(bytes: Bytes) -> Uuid {
        Uuid(bytes)
    }

    /// Creates a UUID using the supplied bytes in little endian order.
    ///
    /// The individual fields encoded in the buffer will be flipped.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # fn main() -> Result<(), uuid::Error> {
    /// # use uuid::Uuid;
    /// let bytes = [
    ///     0xa1, 0xa2, 0xa3, 0xa4,
    ///     0xb1, 0xb2,
    ///     0xc1, 0xc2,
    ///     0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
    /// ];
    ///
    /// let uuid = Uuid::from_bytes_le(bytes);
    ///
    /// assert_eq!(
    ///     "a4a3a2a1-b2b1-c2c1-d1d2-d3d4d5d6d7d8",
    ///     uuid.hyphenated().to_string(),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub const fn from_bytes_le(b: Bytes) -> Uuid {
        Uuid([
            b[3], b[2], b[1], b[0], b[5], b[4], b[7], b[6], b[8], b[9], b[10], b[11], b[12], b[13],
            b[14], b[15],
        ])
    }

    /// Creates a reference to a UUID from a reference to the supplied bytes.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # fn main() -> Result<(), uuid::Error> {
    /// # use uuid::Uuid;
    /// let bytes = [
    ///     0xa1, 0xa2, 0xa3, 0xa4,
    ///     0xb1, 0xb2,
    ///     0xc1, 0xc2,
    ///     0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
    /// ];
    ///
    /// let uuid = Uuid::from_bytes_ref(&bytes);
    ///
    /// assert_eq!(
    ///     uuid.hyphenated().to_string(),
    ///     "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"
    /// );
    ///
    /// assert!(std::ptr::eq(
    ///     uuid as *const Uuid as *const u8,
    ///     &bytes as *const [u8; 16] as *const u8,
    /// ));
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_bytes_ref(bytes: &Bytes) -> &Uuid {
        // SAFETY: `Bytes` and `Uuid` have the same ABI
        unsafe { &*(bytes as *const Bytes as *const Uuid) }
    }

    // NOTE: There is no `from_u128_ref` because in little-endian
    // environments the value isn't properly encoded. Callers would
    // need to use `.to_be()` themselves.
}

impl Builder {
    /// Creates a `Builder` using the supplied bytes.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::Builder;
    /// let bytes = [
    ///     0xa1, 0xa2, 0xa3, 0xa4,
    ///     0xb1, 0xb2,
    ///     0xc1, 0xc2,
    ///     0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
    /// ];
    ///
    /// let uuid = Builder::from_bytes(bytes).into_uuid();
    ///
    /// assert_eq!(
    ///     "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8",
    ///     uuid.hyphenated().to_string(),
    /// );
    /// ```
    pub const fn from_bytes(b: Bytes) -> Self {
        Builder(Uuid::from_bytes(b))
    }

    /// Creates a `Builder` using the supplied bytes in little endian order.
    ///
    /// The individual fields encoded in the buffer will be flipped.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # fn main() -> Result<(), uuid::Error> {
    /// # use uuid::{Builder, Uuid};
    /// let bytes = [
    ///     0xa1, 0xa2, 0xa3, 0xa4,
    ///     0xb1, 0xb2,
    ///     0xc1, 0xc2,
    ///     0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
    /// ];
    ///
    /// let uuid = Builder::from_bytes_le(bytes).into_uuid();
    ///
    /// assert_eq!(
    ///     "a4a3a2a1-b2b1-c2c1-d1d2-d3d4d5d6d7d8",
    ///     uuid.hyphenated().to_string(),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub const fn from_bytes_le(b: Bytes) -> Self {
        Builder(Uuid::from_bytes_le(b))
    }

    /// Creates a `Builder` for a version 1 UUID using the supplied timestamp and node id.
    pub const fn from_rfc4122_timestamp(ticks: u64, counter: u16, node_id: &[u8; 6]) -> Self {
        let time_low = (ticks & 0xFFFF_FFFF) as u32;
        let time_mid = ((ticks >> 32) & 0xFFFF) as u16;
        let time_high_and_version = (((ticks >> 48) & 0x0FFF) as u16) | (1 << 12);

        let mut d4 = [0; 8];

        d4[0] = (((counter & 0x3F00) >> 8) as u8) | 0x80;
        d4[1] = (counter & 0xFF) as u8;
        d4[2] = node_id[0];
        d4[3] = node_id[1];
        d4[4] = node_id[2];
        d4[5] = node_id[3];
        d4[6] = node_id[4];
        d4[7] = node_id[5];

        Self::from_fields(time_low, time_mid, time_high_and_version, &d4)
    }

    /// Creates a `Builder` for a version 6 UUID using the supplied timestamp and node id.
    pub const fn from_sorted_rfc4122_timestamp(ticks: u64, counter: u16, node_id: &[u8; 6]) -> Self {
        let time_high = ((ticks >> 28) & 0xFFFF_FFFF) as u32;
        let time_mid = ((ticks >> 12) & 0xFFFF) as u16;
        let time_low_and_version = ((ticks & 0x0FFF) as u16) | (0x6 << 12);

        let mut d4 = [0; 8];

        d4[0] = (((counter & 0x3F00) >> 8) as u8) | 0x80;
        d4[1] = (counter & 0xFF) as u8;
        d4[2] = node_id[0];
        d4[3] = node_id[1];
        d4[4] = node_id[2];
        d4[5] = node_id[3];
        d4[6] = node_id[4];
        d4[7] = node_id[5];

        Self::from_fields(time_high, time_mid, time_low_and_version, &d4)
    }

    /// Creates a `Builder` for a version 7 UUID using the supplied Unix timestamp in millisecond precision and random data.
    pub const fn from_timestamp_millis(since_unix_epoch: Duration, random_bytes: &[u8; 11]) -> Self {
        let millis = since_unix_epoch.as_millis() as u64;
        let ms_high = ((millis >> 16) & 0xFFFF_FFFF) as u32;
        let ms_low = (millis & 0xFFFF) as u16;

        let rng_ver = random_bytes[0] as u16 | ((random_bytes[1] as u16) << 8) | (0x7 << 12);

        let mut rng_rest = [0; 8];
        rng_rest[0] = (random_bytes[2] & 0x3F) | 0x80;
        rng_rest[1] = random_bytes[3];
        rng_rest[2] = random_bytes[4];
        rng_rest[3] = random_bytes[5];
        rng_rest[4] = random_bytes[6];
        rng_rest[5] = random_bytes[7];
        rng_rest[6] = random_bytes[8];
        rng_rest[7] = random_bytes[9];

        Self::from_fields(ms_high, ms_low, rng_ver, &rng_rest)
    }

    /// Creates a `Builder` for a version 8 UUID using the supplied user-defined bytes.
    pub const fn from_custom_bytes(b: Bytes) -> Self {
        Builder::from_bytes(b)
            .with_variant(Variant::RFC4122)
            .with_version(Version::Custom)
    }

    /// Creates a `Builder` for a version 4 UUID using the supplied random bytes.
    ///
    /// This method can be useful in environments where the `v4` feature isn't
    /// available. This method will take care of setting the appropriate
    /// version and variant fields.
    ///
    /// # Examples
    ///
    /// ```
    /// # use uuid::{Builder, Variant, Version};
    /// # let rng = || [
    /// #     70, 235, 208, 238, 14, 109, 67, 201, 185, 13, 204, 195, 90,
    /// # 145, 63, 62,
    /// # ];
    /// let random_bytes = rng();
    /// let uuid = Builder::from_random_bytes(random_bytes).into_uuid();
    ///
    /// assert_eq!(Some(Version::Random), uuid.get_version());
    /// assert_eq!(Variant::RFC4122, uuid.get_variant());
    /// ```
    pub const fn from_random_bytes(b: Bytes) -> Self {
        Builder(Uuid::from_bytes(b))
            .with_variant(Variant::RFC4122)
            .with_version(Version::Random)
    }

    /// Creates a `Builder` for a version 3 UUID using the supplied MD5 hashed bytes.
    pub const fn from_md5_bytes(b: Bytes) -> Self {
        Builder(Uuid::from_bytes(b))
            .with_variant(Variant::RFC4122)
            .with_version(Version::Md5)
    }

    /// Creates a `Builder` for a version 5 UUID using the supplied SHA1 hashed bytes.
    pub const fn from_sha1_bytes(b: Bytes) -> Self {
        Builder(Uuid::from_bytes(b))
            .with_variant(Variant::RFC4122)
            .with_version(Version::Sha1)
    }

    /// Creates a `Builder` using the supplied bytes.
    ///
    /// # Errors
    ///
    /// This function will return an error if `b` has any length other than 16.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::Builder;
    /// # fn main() -> Result<(), uuid::Error> {
    /// let bytes = [
    ///     0xa1, 0xa2, 0xa3, 0xa4,
    ///     0xb1, 0xb2,
    ///     0xc1, 0xc2,
    ///     0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
    /// ];
    ///
    /// let uuid = Builder::from_slice(&bytes)?.into_uuid();
    ///
    /// assert_eq!(
    ///     "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8",
    ///     uuid.hyphenated().to_string(),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_slice(b: &[u8]) -> Result<Self, Error> {
        Ok(Builder(Uuid::from_slice(b)?))
    }

    /// Creates a `Builder` using the supplied bytes in little endian order.
    ///
    /// The individual fields encoded in the buffer will be flipped.
    ///
    /// # Errors
    ///
    /// This function will return an error if `b` has any length other than 16.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::Builder;
    /// # fn main() -> Result<(), uuid::Error> {
    /// let bytes = [
    ///     0xa1, 0xa2, 0xa3, 0xa4,
    ///     0xb1, 0xb2,
    ///     0xc1, 0xc2,
    ///     0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
    /// ];
    ///
    /// let uuid = Builder::from_slice_le(&bytes)?.into_uuid();
    ///
    /// assert_eq!(
    ///     "a4a3a2a1-b2b1-c2c1-d1d2-d3d4d5d6d7d8",
    ///     uuid.hyphenated().to_string(),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_slice_le(b: &[u8]) -> Result<Self, Error> {
        Ok(Builder(Uuid::from_slice_le(b)?))
    }

    /// Creates a `Builder` from four field values.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::Builder;
    /// let d1 = 0xa1a2a3a4;
    /// let d2 = 0xb1b2;
    /// let d3 = 0xc1c2;
    /// let d4 = [0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8];
    ///
    /// let uuid = Builder::from_fields(d1, d2, d3, &d4).into_uuid();
    ///
    /// assert_eq!(
    ///     uuid.hyphenated().to_string(),
    ///     "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"
    /// );
    /// ```
    pub const fn from_fields(d1: u32, d2: u16, d3: u16, d4: &[u8; 8]) -> Self {
        Builder(Uuid::from_fields(d1, d2, d3, d4))
    }

    /// Creates a `Builder` from four field values.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::Builder;
    /// let d1 = 0xa1a2a3a4;
    /// let d2 = 0xb1b2;
    /// let d3 = 0xc1c2;
    /// let d4 = [0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8];
    ///
    /// let uuid = Builder::from_fields_le(d1, d2, d3, &d4).into_uuid();
    ///
    /// assert_eq!(
    ///     uuid.hyphenated().to_string(),
    ///     "a4a3a2a1-b2b1-c2c1-d1d2-d3d4d5d6d7d8"
    /// );
    /// ```
    pub const fn from_fields_le(d1: u32, d2: u16, d3: u16, d4: &[u8; 8]) -> Self {
        Builder(Uuid::from_fields_le(d1, d2, d3, d4))
    }

    /// Creates a `Builder` from a 128bit value.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::Builder;
    /// let v = 0xa1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8u128;
    ///
    /// let uuid = Builder::from_u128(v).into_uuid();
    ///
    /// assert_eq!(
    ///     "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8",
    ///     uuid.hyphenated().to_string(),
    /// );
    /// ```
    pub const fn from_u128(v: u128) -> Self {
        Builder(Uuid::from_u128(v))
    }

    /// Creates a UUID from a 128bit value in little-endian order.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::Builder;
    /// let v = 0xa1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8u128;
    ///
    /// let uuid = Builder::from_u128_le(v).into_uuid();
    ///
    /// assert_eq!(
    ///     "d8d7d6d5-d4d3-d2d1-c2c1-b2b1a4a3a2a1",
    ///     uuid.hyphenated().to_string(),
    /// );
    /// ```
    pub const fn from_u128_le(v: u128) -> Self {
        Builder(Uuid::from_u128_le(v))
    }

    /// Creates a `Builder` with an initial [`Uuid::nil`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::Builder;
    /// let uuid = Builder::nil().into_uuid();
    ///
    /// assert_eq!(
    ///     "00000000-0000-0000-0000-000000000000",
    ///     uuid.hyphenated().to_string(),
    /// );
    /// ```
    pub const fn nil() -> Self {
        Builder(Uuid::nil())
    }

    /// Specifies the variant of the UUID.
    pub fn set_variant(&mut self, v: Variant) -> &mut Self {
        *self = Builder(self.0).with_variant(v);
        self
    }

    /// Specifies the variant of the UUID.
    pub const fn with_variant(mut self, v: Variant) -> Self {
        let byte = (self.0).0[8];

        (self.0).0[8] = match v {
            Variant::NCS => byte & 0x7f,
            Variant::RFC4122 => (byte & 0x3f) | 0x80,
            Variant::Microsoft => (byte & 0x1f) | 0xc0,
            Variant::Future => byte | 0xe0,
        };

        self
    }

    /// Specifies the version number of the UUID.
    pub fn set_version(&mut self, v: Version) -> &mut Self {
        *self = Builder(self.0).with_version(v);
        self
    }

    /// Specifies the version number of the UUID.
    pub const fn with_version(mut self, v: Version) -> Self {
        (self.0).0[6] = ((self.0).0[6] & 0x0f) | ((v as u8) << 4);

        self
    }

    /// Get a reference to the underlying [`Uuid`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::Builder;
    /// let builder = Builder::nil();
    ///
    /// let uuid1 = builder.as_uuid();
    /// let uuid2 = builder.as_uuid();
    ///
    /// assert_eq!(uuid1, uuid2);
    /// ```
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Convert the builder into a [`Uuid`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::Builder;
    /// let uuid = Builder::nil().into_uuid();
    ///
    /// assert_eq!(
    ///     uuid.hyphenated().to_string(),
    ///     "00000000-0000-0000-0000-000000000000"
    /// );
    /// ```
    pub const fn into_uuid(self) -> Uuid {
        self.0
    }
}
