pub fn start_db() {
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
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