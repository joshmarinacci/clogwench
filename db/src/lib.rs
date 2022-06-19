use serde::{Deserialize, Serialize};
use serde_json::{Value, Result};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::iter::{Filter, Iterator, Map};
use std::path::PathBuf;
use std::slice::Iter;

pub struct JDB {
    pub data: Vec<JObj>,
}

impl JDB {
    pub fn process_query(&self, req: HashMap<String,String>) -> Vec<JObj> {
        println!("db processing the query {:?}",req);
        let mut results:Vec<JObj> = vec![];
        for item in &self.data {
            for (key,value) in &req {
                println!("searching for the key '{}'",key);
                if item.field_matches(&key, &value) {
                    results.push(item.clone())
                }
            }
        }
        return results;
    }
    pub fn load_from_file(filepath: PathBuf) -> JDB {
        println!("Loading {:?}",filepath.canonicalize().unwrap());
        let file = File::open(filepath).unwrap();
        let val:Value = serde_json::from_reader(BufReader::new(file)).unwrap();
        // println!("value is {}",val);
        let objs = val.as_object().unwrap().get("data").unwrap();
        // println!("objects are {}",objs);
        let mut jdb = JDB {
            data: vec![]
        };
        for ob in objs.as_array().unwrap() {
            // println!("object {:?}",ob);
            let mut song = JObj::make();
            // ob.as_object().
            let id = ob.get("id").unwrap();
            song.id = id.as_str().unwrap().to_string();
            let mp = ob.get("data").unwrap().as_object().unwrap();
            for (s,v) in mp.iter() {
                // println!("key {} value {}",s,v);
                song.data.insert(s.clone(), v.as_str().unwrap().to_string());
            }
            println!("adding a db object {:?}",song);
            jdb.data.push(song);
        }
        return jdb
    }

    pub(crate) fn find_by_field(&self, name: &str, value: &str) -> Vec<&JObj> {
        self.data.iter().filter(|o|o.field_matches(name,value)).collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JObj {
    pub id:String,
    pub data:HashMap<String,String>
}

impl JObj {
    pub fn make() -> JObj {
        JObj {
            id:String::default(),
            data: Default::default()
        }
    }

    pub(crate) fn field(&self, name: &str) -> Option<&String> {
        self.data.get(name)
    }

    fn has_field(&self, field_name: &str) -> bool {
        return self.data.contains_key(field_name)
    }

    fn field_matches(&self, name:&str, value:&str) -> bool {
        if let Some(val) = self.data.get(name) {
            println!("comparing {} and {}",&val,value.to_string());
            return val.eq(value)
        } else {
            return false;
        }

    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::env;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::PathBuf;
    use serde::de::Error;
    use serde_json::Value;
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


        let str = serde_json::to_string_pretty(&objs[0]).unwrap();
        println!("generated {}",str);


        let data = r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#;
        // let res:Value = serde_json::from_str(data).unwrap();
        // println!("value is {}",res);
        // serde_json::to_writer_pretty(file, &res).unwrap();

    }

    #[test]
    fn load_file_test() {
        println!("working dir is {:?}", env::current_dir());
        let jdb = JDB::load_from_file(PathBuf::from("./test_data.json"));
        assert_eq!(jdb.data.len(),3);
        let mut query:HashMap<String,String> = HashMap::new();
        query.insert(String::from("type"), String::from("song-track"));
        let res = jdb.process_query(query);
        assert_eq!(res.len(),3);
    }

    fn make_test_db() -> JDB {
        let mut jdb = JDB {
            data: vec![]
        };
        let mut song = JObj::make();
        song.data.insert("title".to_string(), "Catch Me I'm Falling".to_string());
        song.data.insert("artist".to_string(), "Pretty Poison".to_string());
        song.data.insert("album".to_string(), "Catch Me I'm Falling".to_string());
        jdb.data.push(song);

        let mut song = JObj::make();
        song.data.insert("title".to_string(), "Nightime".to_string());
        song.data.insert("artist".to_string(), "Pretty Poison".to_string());
        song.data.insert("album".to_string(), "Catch Me I'm Falling".to_string());
        jdb.data.push(song);

        let mut song = JObj::make();
        song.data.insert("title".to_string(), "Closer".to_string());
        song.data.insert("artist".to_string(), "Pretty Poison".to_string());
        song.data.insert("album".to_string(), "Catch Me I'm Falling".to_string());
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