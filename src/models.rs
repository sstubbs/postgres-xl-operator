// Generated by diesel_ext

#![allow(unused)]
#![allow(clippy::all)]


use chrono::offset::Utc;
use chrono::DateTime;

#[derive(Queryable, Debug)]
pub struct Ping {
    pub id: Option<i32>,
    pub t_ins: Option<DateTime<Utc>>,
}
