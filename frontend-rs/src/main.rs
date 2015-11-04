#![feature(plugin)]
#![feature(unsafe_destructor)]
#![plugin(concat_bytes)]

#[macro_use] extern crate webplatform;
extern crate libc;

static counter : u32 = 0;

use webplatform::{Date, Document, SessionStorage};

use std::borrow::ToOwned;
use std::str::FromStr;

fn get_time(document: Document) {
    run();

    let clock = document.element_query("#clock").unwrap();
    clock.style_set_str("visibility", "visible");
    clock.html_set("00 : 00 : 00");

    let now = Date::now();
    SessionStorage.set("start", &now.to_string());
}

fn stop_time(document: Document) {
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
    jquery.post("api/time/new", &data, |data| {
        load_dom(&document);
    });
}

fn toggleTimer(document: Document) {
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
}

fn load_dom(document: &Document) {
    let jquery = webplatform::JQuery::new();

    jquery.ajax("http://localhost:3000/api/time", move |data| {
        document.element_query("#timeList").and_then(|t| Some(t.html_set("")));
        js!{ (&data[..]) br#"
            alert("loaded dom");
            let tracks = JSON.parse(UTF8ToString($0));
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
        let body = document.element_query("body").unwrap();

        let hr = document.element_create("hr").unwrap();
        body.append(&hr);

        body.html_prepend("<h1>HELLO FROM RUST</h1>");
        body.html_append("<button>CLICK ME</button>");

        let mut button = document.element_query("button").unwrap();

        let bodyref = body.root_ref();
        let bodyref2 = body.root_ref();
        button.on("click", move |_| {
            bodyref2.prop_set_str("bgColor", "blue");
        });

        let jquery = webplatform::JQuery::new();

        jquery.ajax("/webplatform.html", move |data| {
            println!("ajax executed!, data: {:?}", data);
        });

        webplatform::SessionStorageInterface.set("start", "0");

        load_dom(&document);

        println!("This should be blue: {:?}", bodyref.prop_get_str("bgColor"));
        println!("Width?: {:?}", bodyref.prop_get_i32("clientWidth"));
        println!("Timestamp: {:?}", webplatform::Date::now());

        webplatform::spin();
    }

    println!("NO CALLING ME.");
}
