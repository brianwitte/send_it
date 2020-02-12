#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use std::io::Read;
use std::fmt;

use base64::decode;

use rocket::{Request, Data, Outcome, Outcome::*};
use rocket::data::{self, FromDataSimple};
use rocket::http::{Status, ContentType};


//#[derive(Debug)]
struct Login {
    username: String,
    password: String
}

const LIMIT: u64 = 256;

impl FromDataSimple for Login {
    type Error = String;

    fn from_data(req: &Request, data: Data) -> data::Outcome<Self, String> {
        // Ensure the content type is correct before opening the data.
        let login_ct = ContentType::new("Authorization", "Basic");
        if req.content_type() != Some(&login_ct) {
            return Outcome::Forward(data);
        }

	data = base64::decode(data).unwrap();

        // Read the data into a String.
        let mut string = String::new();
        if let Err(e) = data.open().take(LIMIT).read_to_string(&mut string) {
            return Failure((Status::InternalServerError, format!("{:?}", e)));
        }

        // Split the string into two pieces at ':'.
        let (username, password) = match string.find('&') {
            Some(i) => (string[..i].to_string(), &string[(i + 1)..]),
            None => return Failure((Status::UnprocessableEntity, "':'".into()))
        };

        // Parse the password.
        let password: String = match password.parse() {
            Ok(password) => password,
            Err(_) => return Failure((Status::UnprocessableEntity, "password".into()))
        };

        // Return successfully.
        Success(Login {username, password})
    }
}

impl fmt::Display for Login {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "username: {}, password: {}", self.username, self.password)
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

// curl -d "uNiqUE_User1337:specialpassword5000" -H "Content-Type: application/x-login" -X POST http://localhost:8000/api/testpost
#[post("/testpost", format = "Authorization/Basic", data = "<input>")]
fn test_login_post(input: Login) -> String {
    format!("http POST -> the input here is {}\n", input)
}

fn main() {

    let api_routes = routes![
        index,
        test_login_post
    ];

    rocket::ignite()
        .mount("/api", api_routes).launch();
}
