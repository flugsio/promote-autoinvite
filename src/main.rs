#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use std::env;

use rocket::request::Form;
use rocket::response::content::Html;
use rocket::response::Redirect;

use std::collections::HashMap;

#[derive(FromForm)]
struct Membership {
    email: String,
    first_name: String,
    last_name: String,
}

#[get("/")]
fn index() -> Html<&'static str> {
        Html("<a href='https://github.com/flugsio/promote-autoinvite/blob/master/src/main.rs'>Check the source code here</a><h3>Promote self registration</h3><form method=post action=/membership>Email: <input type=text name=email><br>First name: <input type=text name=first_name><br>Last name: <input type=text name=last_name><br><input type=submit></form>")
}

#[post("/membership", data = "<membership>")]
fn new(membership: Form<Membership>) -> Redirect {
    let mut api = PromoteAPI::new();

    let auth = api.create_auth();

    api.create_user(membership.email.clone(), membership.first_name.clone(), membership.last_name.clone());

    api.create_membership(membership.email.clone());
    let invitations = api.create_invitation(membership.email.clone());
    Redirect::to(invitations.result[0].url.clone())
}

#[derive(serde::Deserialize)]
struct PromoteToken {
    access_token: String,
    //refresh_token: String,
    //token_type: String,
    //expires_in: i64,
    //scope: String,
    //user_id: String,
    //audience: String,
}

#[derive(serde::Deserialize)]
struct Invitations {
    result: Vec<Invitation>,
}

#[derive(serde::Deserialize)]
struct Invitation {
    id: String,
    url: String,
    program: String,
    user: String,
}

struct PromoteAPI {
    token: PromoteToken,
}

impl PromoteAPI {
    pub fn new() -> PromoteAPI {
        PromoteAPI {
            token: PromoteToken { access_token: "".to_string() },
        }
    }

    fn create_auth(&mut self) {
        let username = env::var("API_USERNAME").unwrap();
        let password = env::var("API_PASSWORD").unwrap();
        let client_id = env::var("API_CLIENT_ID").unwrap();
        let auth_server = env::var("AUTH_SERVER").unwrap();

        let mut data = HashMap::new();
        data.insert("grant_type", "password");
        data.insert("username", &username);
        data.insert("password", &password);
        data.insert("client_id", &client_id);

        let client = reqwest::blocking::Client::new();
        let res = client.post(format!("{}/oauth/token", auth_server))
            .json(&data)
            .send()
            .unwrap()
            .json::<PromoteToken>();
        self.token = res.unwrap();
    }

    pub fn create_user(&self, email: String, first_name: String, last_name: String) {
        let server = env::var("PROMOTE_SERVER").unwrap();
        let mut data = HashMap::new();
        data.insert("email", email);
        data.insert("first_name", first_name);
        data.insert("last_name", last_name);

        let client = reqwest::blocking::Client::new();
        let res = client.post(format!("{}/api/users", server))
            .json(&data)
            .header("Accept-Version", "v3")
            .header("Authorization", format!("Bearer {}", self.token.access_token))
            .send();
    }

    pub fn create_membership(&self, email: String) {
        let server = env::var("PROMOTE_SERVER").unwrap();
        let program_uuid = env::var("PROMOTE_PROGRAM_UUID").unwrap();
        let data = [("user", email), ("roles[]", "learner".to_string())];

        let client = reqwest::blocking::Client::new();
        let res = client.post(format!("{}/api/programs/{}/members", server, program_uuid))
            .form(&data)
            .header("Accept-Version", "v3")
            .header("Authorization", format!("Bearer {}", self.token.access_token))
            .send();
        println!("{:?}", res);
    }

    pub fn create_invitation(&self, email: String) -> Invitations {
        let server = env::var("PROMOTE_SERVER").unwrap();
        let program_uuid = env::var("PROMOTE_PROGRAM_UUID").unwrap();
        let data = [("users[]", email)];

        let client = reqwest::blocking::Client::new();
        let res = client.post(format!("{}/api/programs/{}/invitations", server, program_uuid))
            .form(&data)
            .header("Accept-Version", "v3")
            .header("Authorization", format!("Bearer {}", self.token.access_token))
            .send()
            .unwrap()
            .json::<Invitations>();
        res.unwrap()
    }
}

fn main() {
    rocket::ignite().mount("/", routes![index, new]).launch();
}
