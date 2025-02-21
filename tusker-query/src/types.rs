use std::{collections::HashMap, net::IpAddr, time::SystemTime};

/// This is merely a marker interface
pub trait FromSqlTyped<'a, T> {}

/// BOOL
pub struct PgBool;
/// CHAR
pub struct PgI8;
/// SMALLINT, SMALLSERIAL
pub struct PgI16;
/// INT, SERIAL
pub struct PgI32;
/// OID
pub struct PgI64;
/// REAL
pub struct PgF32;
/// DOUBLE PRECISION
pub struct PgF64;
/// VARCHAR, CHAR(n), TEXT, CITEXT, NAME, UNKNOWN, LTREE, LQUERY, LTXTQUERY
pub struct PgString;
/// BYTEA
pub struct PgBytea;
/// HSTORE
pub struct PgHstore;
/// TIMESTAMP
pub struct PgTimestamp;
/// TIMESTAMP WITH TIME ZONE
pub struct PgTimestampTz;
/// INET
pub struct PgInet;
/// DATE
pub struct PgDate;
/// TIME
pub struct PgTime;
/// UUID
pub struct PgUuid;
/// JSON, JSONB
pub struct PgJson;

impl FromSqlTyped<'_, PgBool> for bool {}
impl FromSqlTyped<'_, PgI8> for i8 {}
impl FromSqlTyped<'_, PgI16> for i16 {}
impl FromSqlTyped<'_, PgI32> for i32 {}
impl FromSqlTyped<'_, PgI64> for i64 {}
impl FromSqlTyped<'_, PgF32> for f32 {}
impl FromSqlTyped<'_, PgF64> for f64 {}
impl<'a> FromSqlTyped<'a, PgString> for &'a str {}
impl FromSqlTyped<'_, PgString> for String {}
impl<'a> FromSqlTyped<'a, PgBytea> for &'a [u8] {}
impl FromSqlTyped<'_, PgBytea> for Vec<u8> {}
impl FromSqlTyped<'_, PgHstore> for HashMap<String, Option<String>> {}
impl FromSqlTyped<'_, PgTimestamp> for SystemTime {}
impl FromSqlTyped<'_, PgTimestampTz> for SystemTime {}
impl FromSqlTyped<'_, PgInet> for IpAddr {}

#[cfg(feature = "with-time-0_3")]
impl<'a> FromSqlTyped<'a, PgTimestamp> for time_03::PrimitiveDateTime {}
#[cfg(feature = "with-time-0_3")]
impl<'a> FromSqlTyped<'a, PgTimestampTz> for time_03::OffsetDateTime {}
#[cfg(feature = "with-time-0_3")]
impl<'a> FromSqlTyped<'a, PgDate> for time_03::Date {}
#[cfg(feature = "with-time-0_3")]
impl<'a> FromSqlTyped<'a, PgTime> for time_03::Time {}

#[cfg(feature = "with-serde_json-1")]
impl<'a> FromSqlTyped<'a, PgJson> for serde_json_1::Value {}

#[cfg(feature = "with-uuid-1")]
impl<'a> FromSqlTyped<'a, PgUuid> for uuid_1::Uuid {}
