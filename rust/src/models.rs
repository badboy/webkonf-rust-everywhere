use ohmers::{Reference, Collection};

model!(
    derive { Clone }
    User {
    uniques {
        name : String = "".into();
    };

    tracks : Collection<TimeTrack> = Collection::new();
});

model!(
    derive { Clone }
    TimeTrack {
    start : u32 = 0;
    stop : u32 = 0;

    user : Reference<User> = Reference::new();
});

#[derive(Debug,RustcEncodable)]
pub struct TimeTrackView {
    id: usize,
    start: u32,
    stop: u32
}

impl TimeTrackView {
    pub fn from(track: &TimeTrack) -> TimeTrackView {
        TimeTrackView {
            id: track.id,
            start: track.start,
            stop: track.stop,
        }
    }
}
