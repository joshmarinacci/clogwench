extern crate core;

use serde::{Deserialize, Serialize};
use serde_json::{Value, Result};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::iter::{Filter, Iterator, Map};
use std::path::PathBuf;
use std::slice::Iter;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

pub struct JDB {
    data: Vec<JObj>,
}


impl JDB {
    pub(crate) fn find_by_id(&self, id: &str) -> Option<&JObj> {
        self.data.iter().find(|obj|obj.id == id && obj.deleted==false)
    }
    pub(crate) fn update_object(&mut self, obj: JObj) {
        self.data.retain(|ob| ob.id != obj.id);
        self.data.push(obj);
    }
    pub(crate) fn delete(&mut self, obj: &JObj) {
        if let Some(ob) = self.data.iter_mut().find(|ob|ob.id == obj.id) {
            ob.deleted = true;
        }
    }
}

impl JDB {
    pub fn process_query(&self, query: &JQuery) -> Vec<JObj> {
        // println!("db processing the query");
        let mut results:Vec<JObj> = vec![];
        for item in &self.data {
            if item.deleted == false && query.matches(item) {
                results.push(item.clone());
            }
        }
        // println!("final results are {:?}",results);
        return results;
    }
    pub fn process_update(&mut self, opb: JObj) -> JObj {
        let cl = opb.clone();
        self.update_object(opb);
        return cl
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
            // println!("adding a db object {:?}",song);
            jdb.data.push(song);
        }
        return jdb
    }
    pub fn make_empty() -> JDB {
        JDB {
            data: vec![]
        }
    }
    pub fn process_add(&mut self, obj:JObj) -> JObj {
        let mut cl = obj.clone();
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();

        cl.id = format!("obj_${}",rand_string);
        println!("adding object {:?}",cl);
        self.add_object(cl.clone());
        return cl
    }

    pub fn process_delete(&mut self, obj:JObj) -> JObj {
        let mut cl = obj.clone();
        self.delete(&obj);
        println!("deleting object {:?}",cl);
        return cl
    }

    pub fn add_object(&mut self, obj:JObj) {
        self.data.push(obj);
    }

    pub(crate) fn find_by_field(&self, name: &str, value: &str) -> Vec<&JObj> {
        self.data.iter().filter(|o|o.field_matches(name,value)).collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JObj {
    pub id:String,
    deleted:bool,
    pub data:HashMap<String,String>
}

impl JObj {
    pub fn make() -> JObj {
        JObj {
            id:String::default(),
            data: Default::default(),
            deleted:false,
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
            // println!("comparing {} and {}",&val,value.to_string());
            return val.eq(value)
        } else {
            return false;
        }

    }

    fn add_field(&mut self, key:&str, value:&str) {
        self.data.insert(String::from(key), String::from(value));
    }
    pub(crate) fn set_field(&mut self, key: &str, value: &str) {
        self.data.insert(String::from(key), String::from(value));
    }
    pub(crate) fn remove_field(&mut self, key: &str) {
        self.data.remove(key);
    }
}

#[derive(Debug)]
pub enum JClause {
    equals(String),
    equalsi(String),
    substring(String),
    substringi(String),
}

pub struct JQuery {
    clauses:HashMap<String,JClause>
}


impl JQuery {
    pub fn new() -> JQuery {
        JQuery {
            clauses: Default::default()
        }
    }
}

impl JQuery {
    fn matches(&self, item:&JObj) -> bool {
        // println!("trying to match item {:?}",item);
        for (key,value) in self.clauses.iter() {
            // println!("testing key {} == {:?}",key, value);
            if !item.has_field(key) {
                return false;
            }
            if let Some(val) = item.field(key) {
                match value {
                    JClause::equals(t) => {
                        // println!("equal: comparing {} and {}",t,val);
                        if t == val {
                            // println!("is true");
                            continue;
                        } else {
                            // println!("is not true");
                            return false;
                        }
                    }
                    JClause::equalsi(t) => {
                        // println!("fuzzy: comparing {} and {}",t,val);
                        if t.to_lowercase() == val.to_lowercase() {
                            // println!("is true");
                            continue;
                        } else {
                            // println!("is not true");
                            return false;
                        }
                    }
                    JClause::substring(t) => {
                        if val.contains(t) {
                            continue;
                        } else {
                            return false
                        }
                    }
                    JClause::substringi(t) => {
                        let val = &val.to_lowercase();
                        let t = &t.to_lowercase();
                        if val.contains(t) {
                            continue;
                        } else {
                            return false
                        }
                    }
                }
            } else {
                return false;
            }
        }
        return true
    }
    pub fn add_equal(&mut self, key: &str, value: &str) {
        self.clauses.insert(String::from(key), JClause::equals(String::from(value)));
    }
    pub fn add_equal_ci(&mut self, key: &str, value: &str) {
        self.clauses.insert(String::from(key), JClause::equalsi(String::from(value)));
    }
    pub fn add_substring(&mut self, key: &str, value: &str) {
        self.clauses.insert(String::from(key), JClause::substring(String::from(value)));
    }
    pub fn add_substringi(&mut self, key: &str, value: &str) {
        self.clauses.insert(String::from(key), JClause::substringi(String::from(value)));
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
    use crate::{JDB, JObj, JQuery};

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
        // println!("generated {}",str);


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
        assert_eq!(jdb.data.len(),5);
        let mut query:JQuery = JQuery::new();
        query.add_equal("type","song-track");
        // let mut query:HashMap<String,String> = HashMap::new();
        // query.insert(String::from("type"), String::from("song-track"));
        let res = jdb.process_query(&query);
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

    #[test]
    fn query_test() {
        let jdb = JDB::load_from_file(PathBuf::from("./test_data.json"));
        assert_eq!(jdb.data.len(),5);
        {
            // search for all contacts
            let mut query = JQuery::new();
            query.add_equal("type","person-contact");
            let res = jdb.process_query(&query);
            assert_eq!(res.len(), 2);
        }
        {
            // search for person contacts with first equal to Josh
            let mut query = JQuery::new();
            query.add_equal("type","person-contact");
            query.add_equal("first","Josh");
            let res = jdb.process_query(&query);
            assert_eq!(res.len(), 1);
        }
        {
            // search for person contacts with case insensitive first == josh
            let mut query = JQuery::new();
            query.add_equal("type","person-contact");
            query.add_equal_ci("first", "josh");
            let res = jdb.process_query(&query);
            assert_eq!(res.len(), 1);
        }

        {
            // search for person contacts with case insensitive first contains jo
            let mut query = JQuery::new();
            query.add_equal("type","person-contact");
            query.add_substring("first", "osh");
            let res = jdb.process_query(&query);
            assert_eq!(res.len(), 1);
        }
    }

    #[test]
    fn create_object_test() {
        let mut jdb = JDB::load_from_file(PathBuf::from("./test_data.json"));
        // confirm only 2 contacts
        {

            let mut query = JQuery::new();
            query.add_equal("type","person-contact");
            let res = jdb.process_query(&query);
            assert_eq!(res.len(), 2);
        }
        // insert a new contact
        {
            let mut obj = JObj::make();
            obj.add_field("type","person-contact");
            obj.add_field("first","Waylon");
            obj.add_field("last","Smithers");
            jdb.add_object(obj)
        }
        // confirm now we have 3 contacts
        {
            let mut query = JQuery::new();
            query.add_equal("type","person-contact");
            let res = jdb.process_query(&query);
            assert_eq!(res.len(), 3);
        }

    }

    #[test]
    fn edit_object_test() {
        let mut jdb = JDB::load_from_file(PathBuf::from("./test_data.json"));
        {

            // confirm contact exists
            if let Some(obj) = jdb.find_by_id("some-unique-id-05") {
                let mut obj: JObj = obj.clone();
                //edit the contact
                obj.set_field("first", "Bart");
                obj.set_field("last", "Simpson");
                obj.add_field("animated", "true");
                obj.remove_field("email");
                jdb.update_object(obj)
            } else {
                assert!(false,"couldnt find it anymore");
            }
        }

        // get object again
        {
            if let Some(obj) = jdb.find_by_id("some-unique-id-05") {
                // println!("the object is {:?}",obj);
                assert_eq!(obj.id,"some-unique-id-05");
                assert!(obj.has_field("first"));
                assert!(obj.field_matches("first","Bart"));
                assert!(obj.field_matches("last","Simpson"));
                assert!(obj.field_matches("animated","true"));
                assert!(!obj.has_field("email"));
            } else {
                assert!(false,"couldnt find it anymore");
            }
        }
    }

    #[test]
    fn delete_object_test() {
        let mut jdb = JDB::load_from_file(PathBuf::from("./test_data.json"));
        {
            // confirm contact exists
            if let Some(obj) = jdb.find_by_id("some-unique-id-05") {
                let mut obj: JObj = obj.clone();
                jdb.delete(&obj);
            } else {
                assert!(false,"couldnt find it anymore");
            }
        }

        // get object again
        {
            if let Some(obj) = jdb.find_by_id("some-unique-id-05") {
                println!("the object is {:?}",obj);
                assert!(false,"wasn't deleted!");
            } else {
                assert!(true,"fully deleted");
            }
        }
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
