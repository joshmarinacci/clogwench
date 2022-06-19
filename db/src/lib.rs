/*

track {
    title: string
    artist: string
    album: string
}

"Catch Me I'm Falling", "Pretty Poison", "Catch Me I'm Falling"
"Nightime", "Pretty Poison", "Catch Me I'm Falling"
"Closer", "Pretty Poison", "Catch Me I'm Falling"


 */


use std::collections::HashMap;
use std::iter::{Filter, Iterator, Map};
use std::slice::Iter;

struct JDB {
    data: Vec<JObj>,
}

impl JDB {
    pub(crate) fn find_by_field(&self, name: &str, value: &str) -> Vec<&JObj> {
        self.data.iter().filter(|o|o.field_matches(name,value)).collect()
    }
}

struct JObj {
    fields:HashMap<String,String>
}

impl JObj {
    pub(crate) fn field(&self, name: &str) -> Option<&String> {
        self.fields.get(name)
    }
}


impl JObj {
    fn make() -> JObj {
        JObj {
            fields: Default::default()
        }
    }

    fn has_field(&self, field_name: &str) -> bool {
        return self.fields.contains_key(field_name)
    }

    fn field_matches(&self, name:&str, value:&str) -> bool {
        if let Some(val) = self.fields.get(name) {
            println!("comparing {} and {}",val,value);
            return val.eq(value)
        } else {
            return false;
        }

    }
}

#[cfg(test)]
mod tests {
    use crate::{JDB, JObj};

    #[test]
    fn it_works() {
        let db = make_test_db();

        let objs = db.find_by_field("title","Catch Me I'm Falling");
        assert_eq!(objs.len(),1);

        let objs = db.find_by_field("title","madeup");
        assert_eq!(objs.len(),0);

        let objs = db.find_by_field("artist","Pretty Poison");
        assert_eq!(objs.len(),3);

        let objs = db.find_by_field("title","Catch Me I'm Falling");
        assert_eq!(objs.len(),1);
        assert_eq!(objs[0].field("title").unwrap(),"Catch Me I'm Falling");
        assert_eq!(objs[0].field("artist").unwrap(),"Pretty Poison");
        assert_eq!(objs[0].field("album").unwrap(),"Catch Me I'm Falling");

    }

    fn make_test_db() -> JDB {
        let mut jdb = JDB {
            data: vec![]
        };
        let mut song = JObj::make();
        song.fields.insert("title".to_string(), "Catch Me I'm Falling".to_string());
        song.fields.insert("artist".to_string(), "Pretty Poison".to_string());
        song.fields.insert("album".to_string(), "Catch Me I'm Falling".to_string());
        jdb.data.push(song);

        let mut song = JObj::make();
        song.fields.insert("title".to_string(), "Nightime".to_string());
        song.fields.insert("artist".to_string(), "Pretty Poison".to_string());
        song.fields.insert("album".to_string(), "Catch Me I'm Falling".to_string());
        jdb.data.push(song);

        let mut song = JObj::make();
        song.fields.insert("title".to_string(), "Closer".to_string());
        song.fields.insert("artist".to_string(), "Pretty Poison".to_string());
        song.fields.insert("album".to_string(), "Catch Me I'm Falling".to_string());
        jdb.data.push(song);

       return jdb
    }
}


/*
Build simple rust db with unit tests.
- create db service, same process
- create
	- create contact object
	- query contact object
	- delete contact object
- create five contact objects
	- query to get all five
	- query to get just three of them
	- delete them all
	- query to see none are left
	- shut down
- init db from test JSON file
	- query the contact objects
	- add a new object
	- query the objects again to see the new one
	- shut down
- live updates
	- init db from test JSON file
	- create a live query object for the contacts
	- create a new object
	- receive update that query has changed
	- get new set of contacts
	- shut down
 */