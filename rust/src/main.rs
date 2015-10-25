extern crate iron;
extern crate persistent;
extern crate cookie;
extern crate oatmeal_raisin;
#[macro_use]
extern crate router;

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate ohmers;
extern crate rustc_serialize;

extern crate redis;
extern crate r2d2;
extern crate r2d2_redis;
extern crate rand;

use iron::prelude::*;
use iron::status;
use iron::typemap::Key;
use iron::middleware;
use router::Router;
use persistent::Read;
use oatmeal_raisin::{Cookie, CookieJar, SetCookie, SigningKey};

use std::ops::Deref;
use r2d2::Pool;
use r2d2_redis::{RedisConnectionManager};
use redis::Commands;
use ohmers::{Ohmer, OhmerError, with, get};
use rustc_serialize::json;

use std::error::Error;
use std::fmt::{self, Debug};

use rand::{thread_rng, Rng};

mod models;
use models::{User, TimeTrack, TimeTrackView};

pub type RedisPool = Pool<RedisConnectionManager>;
pub struct AppDb;
impl Key for AppDb { type Value = RedisPool; }

struct UserFetch;

impl UserFetch {
    pub fn both() -> (UserFetch,UserFetch) {
        (UserFetch, UserFetch)
    }
}

impl Key for User { type Value = User; }

#[derive(Debug)]
struct StringError(String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for StringError {
    fn description(&self) -> &str { &*self.0 }
}

impl middleware::BeforeMiddleware for UserFetch {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        let pool = req.get::<Read<AppDb>>().unwrap();
        let conn = pool.get().unwrap();

        let user = {
            let cookie_jar = req.get_mut::<CookieJar>().unwrap();
            let sig_jar = cookie_jar.signed();
            fetch_user_or_create(&sig_jar, conn.deref())
        };

        match user {
            None => {
                Err(IronError::new(
                        StringError("Unauthorized".into()),
                        status::Unauthorized))
            }
            Some(user) => {
                req.extensions.insert::<User>(user);
                Ok(())
            }
        }

    }
}

impl middleware::AfterMiddleware for UserFetch {
   fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
       let user = match req.extensions.get::<User>() {
           Some(user) => user.name.clone(),
           None => return Ok(res)
       };

       let cookie_jar = req.get_mut::<CookieJar>().unwrap();
       let sig_jar = cookie_jar.signed();
       sig_jar.add(Cookie::new("user-id".into(), user));
       Ok(res)
   }
}

fn main() {
    env_logger::init().unwrap();
    info!("Starting server");

    let config = r2d2::Config::builder()
        .connection_timeout_ms(2*1000)
        .pool_size(3)
        .build();
    let manager = RedisConnectionManager::new("redis://localhost").unwrap();
    let pool = r2d2::Pool::new(config, manager).unwrap();

    let router = router!(post "/api/time/new" => new_track,
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
    chain.link(UserFetch::both());
    chain.link_after(SetCookie);

    info!("Server starting on http://localhost:3000");
    Iron::new(chain).http("localhost:3000").unwrap();
}

fn fetch_user(name: String, conn: &redis::Connection) -> Option<User> {
    info!("fetching user with {:?}", name);
    let user : User = match with("name", name, conn) {
        Err(_) => return None,
        Ok(None) => return None,
        Ok(Some(u)) => u
    };

    info!("fetch_user {:?}", user);
    Some(user)
}

fn new_random_user(conn: &redis::Connection) -> Option<User> {
    let mut retries = 5;
    while retries > 0 {
        let name: String = thread_rng().gen_ascii_chars().take(10).collect();
        match create!(User { name: name, }, *conn) {
            Ok(user) => return Some(user),
            Err(OhmerError::UniqueIndexViolation(_)) => {
                retries -= 1;
                continue;
            },
            _ => return None
        };

    }

    None
}

fn fetch_user_or_create(jar: &cookie::CookieJar, conn: &redis::Connection) -> Option<User> {
    info!("fetch_user_or_create");
    let user = match jar.find("user-id") {
        None => return new_random_user(conn),
        Some(cookie) => {
            let name = cookie.value;
            fetch_user(name, conn)
        }
    };

    match user {
        None => new_random_user(conn),
        Some(user) => Some(user)
    }
}

fn new_track(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, r#"{"crates": "crates"}"#)))
}

fn unauthorized() -> IronResult<Response> {
    Ok(Response::with((status::Unauthorized, "Unauthorized")))
}
fn notfound() -> IronResult<Response> {
    Ok(Response::with((status::NotFound, "Not found")))
}

fn show_track(req: &mut Request) -> IronResult<Response> {
    let pool = req.get::<Read<AppDb>>().unwrap();
    let conn = pool.get().unwrap();
    let user = req.extensions.get::<User>().unwrap();
    let id = req.extensions.get::<Router>().unwrap().find("id").unwrap_or("");
    let id : usize = ::std::str::FromStr::from_str(id).unwrap_or(0);

    info!("show_track. id={}", id);

    let track : TimeTrack = match get(id, conn.deref()) {
        Err(_) => return notfound(),
        Ok(track) => track
    };

    info!("show_track. user={:?}, track={:?}", user, track);
    let track_user = track.user.get(conn.deref()).unwrap();

    if user.id != track_user.id {
        return unauthorized();
    }

    let track = TimeTrackView::from(&track);
    let encoded = json::encode(&track).unwrap();
    Ok(Response::with((status::Ok, encoded)))
}

fn show_all_tracks(req: &mut Request) -> IronResult<Response> {
    let pool = req.get::<Read<AppDb>>().unwrap();
    let conn = pool.get().unwrap();
    let user = req.extensions.get::<User>().unwrap();

    let user = user.clone();
    let tracks = collection!(user.tracks, *conn.deref())
        .try_into_iter()
        .unwrap()
        .map(|t| TimeTrackView::from(&t))
        .collect::<Vec<_>>();

    info!("tracks: {:?}", tracks);

    let encoded = json::encode(&tracks).unwrap();
    Ok(Response::with((status::Ok, encoded)))
}
