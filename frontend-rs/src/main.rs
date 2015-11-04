#![feature(plugin)]
#![feature(unsafe_destructor)]
#![plugin(concat_bytes)]

#[macro_use] extern crate webplatform;
extern crate libc;

static mut g_counter : u32 = 0;

fn counter(n: u32) -> u32 {
    unsafe { g_counter += n; g_counter }
}

use webplatform::{Date, Document, SessionStorage, log};

use std::borrow::ToOwned;
use std::str::FromStr;

fn get_time(document: &Document) {
    run();

    let clock = document.element_query("#clock").unwrap();
    clock.style_set_str("visibility", "visible");
    clock.html_set("00 : 00 : 00");

    let now = Date::now();
    SessionStorage.set("start", &now.to_string());
}

fn stop_time(document: &Document) {
    log("stop_time start");
    let now = Date::now();
    SessionStorage.set("stop", &now.to_string());
    let clock = document.element_query("#clock").unwrap();
    clock.style_set_str("visibility", "hidden");

    let start = SessionStorage.get("start").unwrap_or("0".to_owned());
    let start = u32::from_str(&start).unwrap_or(0);
    let stop  = SessionStorage.get("stop").unwrap_or("0".to_owned());
    let stop = u32::from_str(&stop).unwrap_or(0);

    let data = format!("start={}&stop={}", start, stop);

    let jquery = webplatform::JQuery::new();
    jquery.post("http://localhost:3000/api/time/new", &data, |_| {
        load_dom(&document);
    });
    log("stop_time end. ajax on its way");
}

fn toggle_timer(document: &Document) {
    log(&format!("toggleTimer started, counter: {}", counter(0)));

    counter(1);
    let track = document.element_query("#track").unwrap();

    if counter(0) % 2 == 0 {
        track.text_set("Go");
        stop_time(document);
        println!("stop_time");
    } else {
        track.text_set("Stop");
        get_time(document);
        println!("get_time");
    }
    log(&format!("toggleTimer ends, counter: {}", counter(0)));
}

fn load_dom(document: &Document) {
    log("loading the dom. I guess.");
    let jquery = webplatform::JQuery::new();

    jquery.ajax("http://localhost:3000/api/time", move |data| {
        document.element_query("#timeList").and_then(|t| Some(t.html_set("")));
        js!{ (&data[..]) br#"
            var tracks = JSON.parse(UTF8ToString($0));
            console.log("got response", tracks);
            for (var i = 0, len = tracks.length; i<len; i++) {
              var start = tracks[i].start * 1000;
              var stop  = tracks[i].stop * 1000;
              var diff = js_formatTime(stop - start);
              $('#timeList').append(
                '<li data-id="' + tracks[i].id + '">' +
                  diff + '</li>'
              );
            }
        "#};
    });
}

fn run() {
    js! {br#"
        if (typeof window.run_timer == "undefined") {
          window.run_timer = function run_timer() {
            var start = parseInt(sessionStorage.getItem('start'), 10) * 1000;
            var now = Date.now();

            document.getElementById('clock').innerHTML = js_formatTime(now-start);
            setTimeout(window.run_timer, 100);
          }
        }

        window.run_timer();
    "#};
}

fn main() {
    let document = webplatform::init();
    {
        let track = document.element_query("#track").unwrap();
        track.on("click", |_e| {
            toggle_timer(&document);
        });
        load_dom(&document);

        webplatform::spin();
    }

    println!("NO CALLING ME.");
}
