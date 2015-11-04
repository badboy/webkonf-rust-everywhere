#![feature(plugin)]
#![feature(unsafe_destructor)]
#![plugin(concat_bytes)]

#[macro_use] extern crate webplatform;
extern crate libc;

static counter : u32 = 0;

use webplatform::{Date, Document, SessionStorage};

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
    jquery.post("http://localhost:3000/api/time/new", &data, |data| {
        load_dom(&document);
    });
}

fn toggleTimer(document: &Document) {
    log(&format!("toggleTimer started, counter: {}", counter));

    counter += 1;
    let track = document.element_query("#track").unwrap();

    if counter % 2 == 0 {
        track.text_set("Go");
        stop_time(document);
        println!("stop_time");
    } else {
        track.text_set("Stop");
        get_time(document);
        println!("get_time");
    }
    log(&format!("toggleTimer ends, counter: {}", counter));
}

fn load_dom(document: &Document) {
    let jquery = webplatform::JQuery::new();

    jquery.ajax("http://localhost:3000/api/time", move |data| {
        document.element_query("#timeList").and_then(|t| Some(t.html_set("")));
        js!{ (&data[..]) br#"
            var tracks = JSON.parse(UTF8ToString($0));
            for (var i = 0, len = tracks.length; i<len; i++) {
              var diff = js_formatTime(tracks[i].stop - tracks[i].start);
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
        if (typeof window.run == "undefined") {
          window.run = function() {
            var start = sessionStorage.getItem('start');
            var now = Date.now();

            document.getElementById('clock').innerHTML = formatTime(now-start);
            setTimeout(run, 100);
          }
        }

        window.run();
    "#};
}

fn main() {
    let document = webplatform::init();
    {
        let track = document.element_query("#track").unwrap();
        track.on("click", |_e| {
            toggleTimer(&document);
        });
        load_dom(&document);

        webplatform::spin();
    }

    println!("NO CALLING ME.");
}
