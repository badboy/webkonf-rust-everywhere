'use strict';

// init hoodie
//window.hoodie   = new Hoodie();

// init timeTracker
window.timeTracker = window.timeTracker || {};

var counter = 0;

var getTime = function getTime(eve){

  // run the clock
  run();

  // reset clock
  document.getElementById('clock').style.visibility = "visible";
  document.getElementById('clock').innerHTML = '00 : 00 : 00';

  // get Time and save it in the sessionStorage so we can refere to it
  var time = Date.now();

  // if there is a start value, we get the diff how much time was spend.
  // we format it and store it in the hoodie.store
  sessionStorage.setItem('start', time);
}

var stopTime = function stopTime(){

  sessionStorage.setItem('stop', Date.now());
  document.getElementById('clock').style.visibility = "hidden";

  var diff;
  var start = sessionStorage.getItem('start');
  var end = sessionStorage.getItem('stop');

  diff = end - start;

  diff = formatTime(diff);

  var watch = { start: start, stop: end };

  $.ajax({
    url: "api/time/new",
    data: watch,
    dataType: "json",
    method: "POST"
  }).done(function (data) {
    loadDOM();
  });
}

var toggleTimer = function toggleTimer(){
  counter++;

  if (counter % 2 == 0) {
    document.getElementById('track').innerText = "Go";
    stopTime();
    console.log('stopTime');
  } else {
    document.getElementById('track').innerText = "Stop";
    getTime();
    console.log('getTime');
  }
}

/* HELPER FUNCTIONs
@desc:   formating the milliseconds in seconds, minutes and hours
@param:  tracked time in milliseconds
*/
var formatTime = function formatTime(ms_time){
  var time = ms_time;
  var s, m, h;

  // when the tracked time is more then an hour (60 * 60 * 1000)
  if (time > 3600000) {
    h = parseInt(time / 3600000);
    time = time % 3600000;
  } else {
    h = 0;
  }

  // when the tracked time is more then a minute (60 * 1000)
  if (time > 60000) {
    m = parseInt(time / 60000);
    time = time % 60000;
  } else {
    m = 0;
  }

  // when the tracked time is more then a second (1000)
  if (time > 1000) {
    s = parseInt(time / 1000);
    time = time % 1000;
  } else {
    s = 0;
  }

  h = addZero(h);
  m = addZero(m);
  s = addZero(s);

  return String(h)+ ' : ' + String(m) + ' : ' + String(s);
}

/*
// loading the data from the hoodie.store in the DOM
*/
var loadDOM = function loadDOM(){
  $.ajax({
    url: "api/time",
    dataType: "json",
  }).done(function (tracks) {
    $('#timeList').empty();
    for (var i = 0, len = tracks.length; i<len; i++) {
      var diff = formatTime(tracks[i].stop - tracks[i].start);
      $('#timeList').append(
        '<li data-id="' + tracks[i].id + '">' +
          diff + '</li>'
      );
    }
  });
}

/*
// running the clock
// the clock shows the time from now to the time we started the timer
// it updates every second and recalls the function,
// so we can show the exact tracked time on the clock
*/
var run = function(){
  var start = sessionStorage.getItem('start');
  var now = Date.now();

  document.getElementById('clock').innerHTML = formatTime(now-start);
  setTimeout(run, 100);
}

/*
// helper to clear the hoodie.store
*/

/*
// helper to make the hour, minute and second a double digit
*/
var addZero = function(n){
  if (n < 10) {
    n = '0' + String(n);
  }
  return n;
}

// deleteDB();

// show the already saved times when page loads
$.ajax({
  url: "api/time/login",
  dataType: "json",
  method: "GET"
}).done(function (data) {
  console.log("login", data);
  loadDOM();
}).fail(function (xhr, status, err) {
  console.log("login failed", {
    xhr: xhr,
    status: status,
    err: err
  });
});

$('#track').bind('click', toggleTimer);
