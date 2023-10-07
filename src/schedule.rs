use chrono::{NaiveDate, Weekday};
use anyhow::Result;
use diesel::{BoolExpressionMethods, ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use crate::models::Course;
use crate::schema::course::dsl::course as course_dsl;
use crate::schema::{Category, course};

#[derive(Debug)]
pub struct Schedule {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub dev_courses: Vec<Course>,
    pub infra_courses: Vec<Course>,
    pub dev_infra_courses: Vec<Course>,
    pub marketing_courses: Vec<Course>,
    pub common_courses: Vec<Course>,
}

impl Schedule {
    pub async fn query_week(date: NaiveDate, connection: &mut MysqlConnection) -> Result<Self> {
        let week = date.week(Weekday::Mon);

        Self::query_between(week.first_day(), week.last_day(), connection).await
    }

    pub async fn query_between(start: NaiveDate, end: NaiveDate, connection: &mut MysqlConnection) -> Result<Self> {
        let dev_courses = course_dsl
            .filter(course::category.eq(Category::Dev)
                .and(course::date.between(start, end)))
            .select(Course::as_select())
            .load(connection)
            .expect("Failed to query dev courses");

        let infra_courses = course_dsl
            .filter(course::category.eq(Category::Infra)
                .and(course::date.between(start, end)))
            .select(Course::as_select())
            .load(connection)
            .expect("Failed to query infra courses");

        let dev_infra_courses = course_dsl
            .filter(course::category.eq(Category::DevInfra)
                .and(course::date.between(start, end)))
            .select(Course::as_select())
            .load(connection)
            .expect("Failed to query dev/infra courses");

        let marketing_courses = course_dsl
            .filter(course::category.eq(Category::Marketing)
                .and(course::date.between(start, end)))
            .select(Course::as_select())
            .load(connection)
            .expect("Failed to query marketing courses");

        let common_courses = course_dsl
            .filter(course::category.eq(Category::Common)
                .and(course::date.between(start, end)))
            .select(Course::as_select())
            .load(connection)
            .expect("Failed to query common courses");

        Ok(Self {
            start,
            end,
            dev_courses,
            infra_courses,
            dev_infra_courses,
            marketing_courses,
            common_courses,
        })
    }
}