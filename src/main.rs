extern crate core;

mod discord;
mod schedule;
mod models;
mod schema;

use std::ops::DerefMut;
use std::sync::Arc;
use axum::{Form, Json, Router};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Html;
use axum::routing::{get, post};
use chrono::{NaiveDateTime};
use diesel::{Connection, MysqlConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use serde::{Deserialize, Serialize};
use tokio::spawn;
use tokio::sync::{Mutex};
use crate::discord::run_discord_bot;
use crate::schedule::Schedule;
use crate::schema::Category;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");
    let db_connection = Arc::new(Mutex::new(
        MysqlConnection::establish(&std::env::var("DATABASE_URL").expect("DATABASE_URL not found"))
            .expect("Failed to connect to database")));

    db_connection.lock().await.run_pending_migrations(MIGRATIONS).expect("Failed to run migrations");

    let web_app = Router::new()
        .route("/", get(index))
        .route("/api/edt", post(edt))
        .with_state(db_connection.clone());

    spawn(async move {
        axum::Server::bind(&"0.0.0.0:80".parse().unwrap())
            .serve(web_app.into_make_service())
            .await
            .unwrap();
    });

    run_discord_bot(db_connection).await.expect("Failed to run discord bot");
}

async fn index() -> Result<Html<String>, StatusCode> {
    Ok(Html(std::fs::read_to_string("index.html").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

#[derive(Serialize)]
struct EndpointResult {
    start: String,
    end: String,
    title: String,
    #[serde(rename = "backgroundColor")]
    background_color: String
}

#[derive(Deserialize, Debug)]
struct EndpointQuery {
    start: String,
    end: String
}

async fn edt(State(state): State<Arc<Mutex<MysqlConnection>>>, Form(params): Form<EndpointQuery>) -> Result<Json<Vec<EndpointResult>>, StatusCode> {
    let start = NaiveDateTime::parse_from_str(&params.start, "%Y-%m-%dT%H:%M:%S").map_err(|_| StatusCode::BAD_REQUEST)?;
    let end = NaiveDateTime::parse_from_str(&params.end, "%Y-%m-%dT%H:%M:%S").map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut schedule = Schedule::query_between(start.date(), end.date(), state.lock().await.deref_mut()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    schedule.dev_courses.append(&mut schedule.infra_courses);
    schedule.dev_courses.append(&mut schedule.dev_infra_courses);
    schedule.dev_courses.append(&mut schedule.common_courses);
    schedule.dev_courses.append(&mut schedule.marketing_courses);

    Ok(Json(
        schedule.dev_courses
            .iter()
            .map(|course| EndpointResult {
                start: format!("{} {}", course.date, course.start),
                end: format!("{} {}", course.date, course.end),
                title: format!("{} {} - {} ({})", if course.bts { "[BTS]" } else { "" }, course.subject, course.teacher, course.classroom),
                background_color: match course.category {
                    Category::Dev => "#007bff",
                    Category::Infra => "#28a745",
                    Category::DevInfra => "#17a2b8",
                    Category::Marketing => "#dc3545",
                    Category::Common => "#ffc107"
                }.to_string()
            })
            .collect()
    ))
}
