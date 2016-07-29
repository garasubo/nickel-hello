#[macro_use]
extern crate nickel;
extern crate rustc_serialize;

#[macro_use(bson, doc)]
extern crate bson;
extern crate mongodb;


use std::collections::HashMap;
// Nickel
use nickel::{Nickel, JsonBody, HttpRouter};
use nickel::mimes::MediaType;
use nickel::status::StatusCode;

// MongoDB
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use mongodb::error::Result as MongoResult;

// bson
use bson::{Bson, Document};
use bson::oid::ObjectId;

// rustc_serialize
use rustc_serialize::json::{Json, ToJson};


#[derive(RustcDecodable, RustcEncodable)]
struct User {
    name: String,
    email: String
}

fn get_data_string(result: MongoResult<Document>) -> Result<Json, String> {
    match result {
        Ok(doc) => Ok(Bson::Document(doc).to_json()),
        Err(e) => Err(format!("{}", e))
    }
}

fn main() {
    let mut serv = Nickel::new();
    let mut router = Nickel::router();

    router.get("/", middleware! {|_, response|
        let mut data = HashMap::new();
        data.insert("color", "Green");
        data.insert("name", "California Apple");
        data.insert("price", "2.50");
        return response.render("assets/hello.tpl", &data);
    });
    router.get("/users", middleware! { |request, mut response|
        // Connect to the database
        let client = Client::connect("localhost", 27017)
            .ok().expect("Error establishing connection.");

        // The users collection
        let coll = client.db("rust-users").collection("users");

        // Create cursor that finds all documents
        let mut cursor = coll.find(None, None).unwrap();

        // Opening for the JSON string to be returned
        let mut data_result = "{\"data\":[".to_owned();

        for (i, result) in cursor.enumerate() {
            match get_data_string(result) {
                Ok(data) => {
                    let string_data = if i == 0 {
                        format!("{}", data)
                    } else {
                        format!("{},", data)
                    };

                    data_result.push_str(&string_data);
                },

                Err(e) => return response.send(format!("{}", e))
            }
        }

        // Close the JSON string
        data_result.push_str("]}");

        // Set the returned type as JSON
        response.set(MediaType::Json);

        // Send back the result
        format!("{}", data_result)
    });
    router.post("/users/new", middleware! { |request, response|
        // Accept a JSON string that corresponds to the User struct
        let user = request.json_as::<User>().unwrap();

        let name = user.name.to_string();
        let email = user.email.to_string();

        // Connect to the database
        let client = Client::connect("localhost", 27017)
            .ok().expect("Error establishing connection.");

        // The users collection
        let coll = client.db("rust-users").collection("users");

        // Insert one user
        match coll.insert_one(doc! {
            "name" => name,
            "email" => email
        }, None) {
            Ok(result) => (match result.write_exception {
                None => (StatusCode::Ok, "Item saved!"),
                Some(e) => (StatusCode::InternalServerError, &*e.message)
            }),
            Err(e) => return response.send(format!("{}", e))
        }
    });
    router.delete("/users/:id", middleware! { |request, response|
        format!("Hello from DELETE /users/:id")
    });

    serv.utilize(router);

    serv.listen("127.0.0.1:6767");
}
