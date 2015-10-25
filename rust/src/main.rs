extern crate iron;
extern crate persistent;
extern crate cookie;
extern crate oatmeal_raisin;
#[macro_use]
extern crate router;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate redis;
extern crate r2d2;
extern crate r2d2_redis;

use iron::prelude::*;
use iron::status;
use iron::typemap::Key;
use router::Router;
use persistent::Read;
use oatmeal_raisin::{Cookie, CookieJar, SetCookie, SigningKey};

use std::ops::Deref;
use r2d2::{Pool, PooledConnection};
use r2d2_redis::{RedisConnectionManager};
use redis::Commands;

mod models;
use models::{User, TimeTrack};

pub type RedisPool = Pool<RedisConnectionManager>;
pub struct AppDb;
impl Key for AppDb { type Value = RedisPool;  }

fn main() {
    env_logger::init().unwrap();
    info!("Starting server");

    let config = r2d2::Config::builder()
        .connection_timeout_ms(2*1000)
        .pool_size(3)
        .build();
    let manager = RedisConnectionManager::new("redis://localhost").unwrap();
    let pool = r2d2::Pool::new(config, manager).unwrap();

    let router = router!(get "/api/time/login" => login,
                         post "/api/time/new" => new_track,
                         get "/api/time/:id" => show_track,
                         get "/api/time" => show_all_tracks);

    let mut chain = Chain::new(router);

    chain.link_before(|req: &mut Request| {
        // Basic logging of requests.
        info!("REQUEST: {}", req.url.path.join("/"));
        Ok(())
    });

    chain.link_before(Read::<SigningKey>::one(b"ba8742af4750"));
    chain.link_before(Read::<AppDb>::one(pool));
    chain.link_after(SetCookie);

    info!("Server starting on http://localhost:3000");
    Iron::new(chain).http("localhost:3000").unwrap();
}

fn fetch_user(name: String) -> Option<User> {
    None
}

fn fetch_user_or_create(jar: &cookie::CookieJar) -> Option<User> {
    match jar.find("user-id") {
        None => return None,
        Some(cookie) => {
            let name = cookie.value;
            return fetch_user(name)
        }
    }
}

fn login(req: &mut Request) -> IronResult<Response> {
    let cookie_jar = req.get_mut::<CookieJar>().unwrap();
    let sig_jar = cookie_jar.signed();

    match fetch_user_or_create(&sig_jar) {
        None => {
            sig_jar.remove("user-id");
            return Ok(Response::with((
                        status::Unauthorized,
                        r#"{"authorized": false, "reason": "No user"}"#
                     )));
        },
        Some(user) => {
            Ok(Response::with((status::Ok, r#"{"crates": "crates"}"#)))
        }
    }

    //let fav = sig_jar.find("favorite");
    //info!("fav: {:?}", fav);
    //sig_jar.add(Cookie::new("favorite".into(), "oatmeal_raisin".into()));
    //Ok(Response::with((status::Ok, r#"{"crates": "crates"}"#)))
}

fn new_track(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, r#"{"crates": "crates"}"#)))
}

fn show_track(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, r#"{"crates": "crates"}"#)))
}

fn show_all_tracks(req: &mut Request) -> IronResult<Response> {
    let pool = req.get::<Read<AppDb>>().unwrap();
    let conn = pool.get().unwrap();

    let res : String = conn.deref().get("foo").unwrap();

    let answer = format!("{{\"crates\": \"{}\"}}", res);
    Ok(Response::with((status::Ok, answer)))
}
