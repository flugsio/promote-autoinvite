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

    api.login();

    api.create_user(&membership.email, &membership.first_name, &membership.last_name);

    api.create_membership(&membership.email);
    let invitations = api.create_invitation(&membership.email);
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
    client: reqwest::blocking::Client,
    server: String,
    token: PromoteToken,
}

impl PromoteAPI {
    pub fn new() -> PromoteAPI {
        PromoteAPI {
            client: reqwest::blocking::Client::new(),
            server: env::var("PROMOTE_SERVER").unwrap(),
            token: PromoteToken { access_token: "".to_string() },
        }
    }

    fn login(&mut self) {
        let client_id = env::var("API_CLIENT_ID").unwrap();
        let client_secret = env::var("API_CLIENT_SECRET").unwrap();
        let auth_server = env::var("PROMOTE_SERVER").unwrap();

        let mut data = HashMap::new();
        data.insert("grant_type", "client_credentials");
        data.insert("client_id", &client_id);
        data.insert("client_secret", &client_secret);
        data.insert("scope", "users:write members:write invitations:write");

        let client = reqwest::blocking::Client::new();
        let res = client.post(format!("{}/oauth/token", auth_server))
            .form(&data)
            .send()
            .unwrap();
        //println!("{:?}", res);
        self.token = res.json::<PromoteToken>().unwrap();
    }

    pub fn create_user(&self, email: &str, first_name: &str, last_name: &str) {
        let data = [("email", email), ("first_name", first_name), ("last_name", last_name)];

        self.api_post("/api/users", &data);
    }

    pub fn create_membership(&self, email: &str) {
        let program_uuid = env::var("PROMOTE_PROGRAM_UUID").unwrap();
        let data = [("user", email), ("roles[]", "learner")];

        self.api_post(&format!("/api/programs/{}/members", program_uuid), &data);
    }

    pub fn create_invitation(&self, email: &str) -> Invitations {
        let program_uuid = env::var("PROMOTE_PROGRAM_UUID").unwrap();
        let data = [("users[]", email)];

        self.api_post(
            &format!("/api/programs/{}/invitations", program_uuid),
            &data
        ).json::<Invitations>().unwrap()
    }

    fn api_post(&self, path: &str, data: &[(&str, &str)]) -> reqwest::blocking::Response {
        self.client.post(format!("{}{}", self.server, path))
            .form(&data)
            .header("Accept-Version", "v3")
            .header("Authorization", format!("Bearer {}", self.token.access_token))
            .send()
            .unwrap()
    }
}

fn main() {
    rocket::ignite().mount("/", routes![index, new]).launch();
}
