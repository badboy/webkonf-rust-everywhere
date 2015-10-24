use std::collections::HashMap;

pub struct User {
    id: u32,
    name: String,
}

pub struct TimeTrack {
    id: u32,
    start: u32,
    stop: u32,
}

pub trait Model {
    fn set_id(&mut self, id: u32);
    fn serialize(&self) -> HashMap<&'static str,String>;
    fn deserialize(data: String) -> Self;
}

impl User {
    pub fn new(name: String) -> User {
        User {
            id: 0,
            name: name
        }
    }
}

impl Model for User {
    fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    fn serialize(&self) -> HashMap<&'static str,String> {
        let mut hash = HashMap::with_capacity(2);
        hash.insert("id", format!("{}", self.id));
        hash.insert("name", self.name.clone());
        hash
    }

    fn deserialize(data: String) -> User {
        User::new("".into())
    }
}

impl TimeTrack {
    pub fn new(start: u32, stop: u32) -> TimeTrack {
        TimeTrack {
            start: start,
            stop: stop,
            id: 0,
        }
    }
}

impl Model for TimeTrack {
    fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    fn serialize(&self) -> HashMap<&'static str,String> {
        let mut hash = HashMap::with_capacity(3);
        hash.insert("id", format!("{}", self.id));
        hash.insert("start", format!("{}", self.start));
        hash.insert("stop", format!("{}", self.stop));
        hash
    }

    fn deserialize(data: String) -> TimeTrack {
        TimeTrack::new(0,0)
    }
}
