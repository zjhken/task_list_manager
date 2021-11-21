#![feature(once_cell)]
#![allow(non_snake_case)]
use chrono::{FixedOffset, NaiveDate};
use seafloor::http_types::{Method, Request, Response};
use seafloor::{
    anyhow::Result, application::App, context::Context, http_types, smol::lock::RwLock,
};
use std::lazy::SyncLazy;
use std::{
    collections::{BTreeMap, HashMap},
    sync::atomic::AtomicU32,
};

pub static TASKS: SyncLazy<RwLock<BTreeMap<u32, (String, NaiveDate)>>> =
    SyncLazy::new(|| RwLock::new(BTreeMap::new()));
pub static ID_SEQ: SyncLazy<AtomicU32> = SyncLazy::new(|| AtomicU32::new(1u32));

fn main() -> Result<()> {
    App::new()
        .setFunc("/add", add)
        .setFunc("/list", list)
        .setFunc("/done", done)
        .start()
}

async fn add(mut ctx: Context) -> Result<Context, http_types::Error> {
    let json = ctx.request.body_json::<HashMap<String, String>>().await?;
    let task = json.get("task").unwrap();
    let date = json.get("date").unwrap();
    println!("task={}", task);
    println!("date={}", date);
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d")?;
    let id = ID_SEQ.fetch_add(1u32, std::sync::atomic::Ordering::SeqCst);
    let mut lockedTasks = TASKS.write().await;
    (*lockedTasks).insert(id, (task.to_owned(), date));
    ctx.response.set_body(format!("{:?}", lockedTasks));
    return Ok(ctx);
}

async fn list(mut ctx: Context) -> Result<Context, http_types::Error> {
    println!("enter list");
    let lockedTasks = TASKS.read().await;
    let body = match ctx.request.body_json::<HashMap<String, String>>().await {
        Ok(json) => {
            let today = json.get("today");
            match today {
                Some(s) => {
                    if s == "Y" {
                        println!("Found request has today param");
                        let now = chrono::Local::now();
                        let today = now.naive_local().date();
                        let x: BTreeMap<_, _> = lockedTasks
                            .iter()
                            .filter(|entry| {
                                let val = entry.1;
                                return val.1.eq(&today);
                            })
                            .collect();
                        println!("{:?}", x);
                        format!("{:?}", x)
                    }
                    else {
                        format!("{:?}", lockedTasks)
                    }
                }
                None => {
                    println!("Found request has no param");
                    format!("{:?}", lockedTasks)
                }
            }
        }
        Err(e) => format!("{:?}", lockedTasks),
    };

    println!("body is {}", body);
    ctx.response.set_body(body);
    return Ok(ctx);
}

async fn done(mut ctx: Context) -> Result<Context, http_types::Error> {
    println!("enter done");
    let mut lockedTasks = TASKS.write().await;
    let body = match ctx.request.body_json::<HashMap<String, u32>>().await {
        Ok(json) => {
            let num = json.get("num");
            match num {
                Some(id) => {
                    (*lockedTasks).retain(|key, _| key != id);
                    format!("{:?}", lockedTasks)
                }
                None => {
                    println!("Found request has no param");
                    format!("{:?}", lockedTasks)
                }
            }
        }
        Err(e) => format!("{:?}", lockedTasks),
    };

    println!("body is {}", body);
    ctx.response.set_body(body);
    return Ok(ctx);
}
