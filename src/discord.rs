/// This module is responsible for the discord bot. It handles slash commands
/// and automatically sends the schedule every week at 6:00 PM on Sunday.

use std::sync::Arc;
use std::ops::DerefMut;

use anyhow::{Result, anyhow};
use chrono::Locale;
use diesel::MysqlConnection;
use poise::command;
use poise::serenity_prelude::{CacheHttp, GatewayIntents};
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use crate::schedule::Schedule;
use crate::schema::Category;

#[derive(Clone)]
struct Data {
    channel: u64,
    admin: u64,
    db_connection: Arc<Mutex<MysqlConnection>>
}

impl Data {
    pub fn new(db_connection: Arc<Mutex<MysqlConnection>>) -> Result<Self> {
        let channel = std::env::var("DISCORD_CHANNEL")
            .map_err(|e| anyhow!("DISCORD_CHANNEL - {}", e))?
            .parse::<u64>()?;
        let admin = std::env::var("DISCORD_ADMIN")
            .map_err(|e| anyhow!("DISCORD_ADMIN - {}", e))?
            .parse::<u64>()?;

        Ok(Self { channel, admin, db_connection })
    }
}

#[command(slash_command)]
async fn send_schedule(ctx: poise::Context<'_, Data, anyhow::Error>) -> Result<(), anyhow::Error> {
    send_week_schedule(&ctx, ctx.data()).await?;
    Ok(())
}

async fn send_week_schedule(cache_http: &impl CacheHttp, data: &Data) -> Result<()> {
    let current_date = chrono::Local::now().date_naive() + chrono::Duration::days(7);
    let schedule = Schedule::query_week(current_date, data.db_connection.lock().await.deref_mut()).await;

    if schedule.is_err() {
        cache_http.http()
            .get_user(data.admin)
            .await?
            .dm(cache_http.http(), |m| {
                m.content(format!("Failed to execute command: {}", schedule.as_ref().unwrap_err()))
            }).await?;

        return Err(anyhow!("Failed to execute command: {}", schedule.unwrap_err()));
    }

    let mut schedule = schedule.unwrap();

    let channel = cache_http.http()
        .get_channel(data.channel).await?
        .guild().ok_or(anyhow!("Channel is not in a guild"))?;

    for message in channel.messages(cache_http.http(), |m| m.limit(20)).await? {
        message.delete(cache_http.http()).await?;
    }

    let mut courses = schedule.dev_courses;
    courses.append(&mut schedule.infra_courses);
    courses.append(&mut schedule.dev_infra_courses);
    courses.append(&mut schedule.common_courses);
    courses.append(&mut schedule.marketing_courses);

    for course in courses {
        channel.send_message(cache_http.http(), |m| {
            m.embed(|embed| {
                embed.title(course.subject)
                    .color(match course.category {
                        Category::Dev => 0x007BFF,
                        Category::Infra => 0x28A745,
                        Category::DevInfra => 0x17A2B8,
                        Category::Marketing => 0xDC3545,
                        Category::Common => 0xFFC107,
                    })
                    .field("Date", format!("{} [{}-{}]", course.date.format_localized("%A", Locale::fr_FR), course.start.format("%H:%M"), course.end.format("%H:%M")), true)
                    .field("Intervenant", course.teacher, true)
                    .field("Salle", course.classroom, true)
                    .field("Spécialité", format!("{}", course.category), true);

                if course.bts {
                    embed.field("BTS", "", true);
                }

                embed
            })
        }).await?;
    }

    Ok(())
}

/// NOTE: Blocking
pub async fn run_discord_bot(db_connection: Arc<Mutex<MysqlConnection>>) -> Result<()> {
    let data = Data::new(db_connection)?;
    let data_clone = data.clone();

    let framework = poise::framework::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![send_schedule()],
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN")?)
        .intents(GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data_clone)
            })
        })
        .build().await?;

    let global_ctx = framework.client().cache_and_http.clone();

    let sched = JobScheduler::new().await?;

    sched.add(Job::new_async("0 0 18 ? * SUN *", move|_, _| {
        let ctx = global_ctx.clone();
        let data = data.clone();
        Box::pin(async move {
            let res = send_week_schedule(&ctx, &data).await;
            if res.is_err() {
                println!("Failed to send schedule: {}", res.unwrap_err());
            }
        })
    })?).await?;

    sched.start().await?;

    framework.start().await?;

    Ok(())
}