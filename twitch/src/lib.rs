#![feature(extern_prelude, attr_literals, nll)]

extern crate rayon;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate failure;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate hyper;
use reqwest::{Client, RequestBuilder};

use std::io::Read;

use clips::{Clips, ClipsOptions};
use failure::Error;
use user::{User, UserIdCache, Users};

header! {(ClientIdHeader, "Client-ID") => [String]}
header! {(Authorization, "Authorization") => [String]}
header! {(AcceptHeader, "Accept") => [String]}

pub mod clips;
pub mod user;

#[derive(Debug, Fail)]
pub enum TwitchError {
    #[fail(display = "Error retrieving clips {}", 0)]
    ClipsError(String),

    #[fail(display = "Error retrieving users {}", 0)]
    RetrievingUsersError(String),

    #[fail(display = "User not found {}", 0)]
    UserNotFoundError(String),
}

pub struct Twitch {
    client_options: ClientOptions,
}

impl Twitch {
    pub fn new(client_options: ClientOptions) -> Self {
        Self { client_options }
    }

    pub fn create_user_id_cache(&mut self) -> UserIdCache {
        UserIdCache::new(self.client_options.clone())
    }

    pub fn get_clips_by_broardcaster(
        &self,
        id: &str,
        options: ClipsOptions,
    ) -> Result<Clips, TwitchError> {
        Clips::from_broadcaster(id, options, &self.client_options)
            .map_err(|err| TwitchError::ClipsError(format!("{}", err).to_string()))
    }

    pub fn get_users_by_name(&self, names: &[&str]) -> Result<Users, TwitchError> {
        Users::by_names(names, &self.client_options)
            .map_err(|err| TwitchError::RetrievingUsersError(format!("{:?}", err)))
    }

    pub fn get_user_by_name(&self, name: &str) -> Result<User, TwitchError> {
        let results = self.get_users_by_name(&[name])?;

        results
            .users
            .into_iter()
            .next()
            .ok_or_else(|| TwitchError::UserNotFoundError(name.to_string()))
    }
}

pub trait TwitchRequestBuilder {
    fn modify_request(&self, client: &mut RequestBuilder);
}

#[derive(Clone)]
pub struct ClientOptions {
    client_id: String,
    bearer: Option<String>,
}

impl ClientOptions {
    pub fn new(id: String, bearer: Option<String>) -> Self {
        Self {
            client_id: id,
            bearer,
        }
    }
}

impl TwitchRequestBuilder for ClientOptions {
    fn modify_request(&self, client: &mut RequestBuilder) {
        client.header(ClientIdHeader(self.client_id.clone()));

        if let Some(ref bearer) = self.bearer.clone() {
            client.header(Authorization(format!("Bearer: {}", bearer.clone())));
        }
    }
}

pub fn send_request(endpoint: &str, client_options: &ClientOptions) -> Result<impl Read, Error> {
    let mut url = String::from("https://api.twitch.tv/");
    url.push_str(endpoint);

    let mut request = Client::new().get(&url);

    client_options.modify_request(&mut request);
    request.header(AcceptHeader("application/vnd.twitchtv.v5+json".to_owned()));

    Ok(request.send()?)
}
