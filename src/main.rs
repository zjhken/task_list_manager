#![feature(once_cell)]
#![allow(non_snake_case)]
#![feature(async_closure)]
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
    println!("[INFO]OPERATION=ADD, task={}, date={}", task, date);
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
                        let map: BTreeMap<_, _> = lockedTasks
                            .iter()
                            .filter(|entry| {
                                let val = entry.1;
                                return val.1.eq(&today);
                            })
                            .collect();
                        println!("{:?}", map);
                        format!("{:?}", map)
                    } else {
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
                    println!("[INFO]OPERATION=DONE, id={}", id);
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

#[cfg(test)]
mod tests {
    use seafloor::http_types::StatusCode;
    use seafloor::smol;

    use super::*;

    #[test]
    fn test_add() {
        smol::block_on(async {
            let mut req = Request::new(Method::Get, "http://example.com");
            req.set_body("{\"task\": \"haha\",\"date\": \"2021-01-01\"}");
            let ctx = Context {
                pathIndex: 1usize,
                request: req,
                response: Response::new(StatusCode::Ok),
                sessionData: Default::default(),
            };
            let body = add(ctx)
                .await
                .unwrap()
                .response
                .body_string()
                .await
                .unwrap();
            assert_eq!(body, "{1: (\"haha\", 2021-01-01)}");
        });
    }
    #[test]
    fn test_list() {
        smol::block_on(async {
            let mut req = Request::new(Method::Get, "http://example.com");
            req.set_body("{\"task\": \"haha\",\"date\": \"2021-01-01\"}");
            let ctx = Context {
                pathIndex: 1usize,
                request: req,
                response: Response::new(StatusCode::Ok),
                sessionData: Default::default(),
            };
            // let ctx = add(ctx).await;
            let body = list(ctx)
                .await
                .unwrap()
                .response
                .body_string()
                .await
                .unwrap();
            assert_eq!(body, "{1: (\"haha\", 2021-01-01)}");
        });
    }

    #[test]
    fn test_done() {
        smol::block_on(async {
            let mut req = Request::new(Method::Get, "http://example.com");
            req.set_body("{\"num\": 1");
            let ctx = Context {
                pathIndex: 1usize,
                request: req,
                response: Response::new(StatusCode::Ok),
                sessionData: Default::default(),
            };
            let body = done(ctx)
                .await
                .unwrap()
                .response
                .body_string()
                .await
                .unwrap();
            assert_eq!(body, "{}");
        });
    }
}
