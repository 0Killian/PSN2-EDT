use chrono::{NaiveDate, NaiveTime};
use diesel::prelude::*;
use crate::schema::Category;

#[derive(Selectable, Queryable, Debug)]
#[diesel(table_name = crate::schema::course)]
pub struct Course {
    pub id: i32,
    pub date: NaiveDate,
    pub category: Category,
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub subject: String,
    pub teacher: String,
    pub classroom: String,
    pub remote: bool,
    pub bts: bool,
}