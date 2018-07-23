use super::send_request;
use failure::Error;
use rayon::prelude::*;
use serde_json;
use std::collections::HashMap;
use std::collections::HashSet;
use ClientOptions;

/// A series of users
#[derive(Serialize, Deserialize, Debug)]
pub struct Users {
    pub users: Vec<User>,
}

/// A single user
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub _id: String,
    pub bio: Option<String>,
    pub created_at: String,
    pub display_name: String,
    pub logo: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub updated_at: String,
}

impl Users {
    /// Get a series of users by their usernames
    pub fn by_names(names: &[&str], client_options: &ClientOptions) -> Result<Users, Error> {
        let mut endpoint = String::from("kraken/users?login=");

        for name in names {
            endpoint.push_str(name);
            endpoint.push(',');
        }

        let endpoint = &endpoint[..endpoint.len() - 1]; // get rid of the extra ',' at the end

        let result: Users = serde_json::from_reader(send_request(endpoint, &client_options)?)?;

        Ok(result)
    }

    /// Get a series of users by their ids.
    pub fn by_ids(ids: &[&str], client_options: &ClientOptions) -> Result<Users, Error> {
        let request_closure = |id_list: &[&str]| {
            let mut endpoint = String::from("kraken/users?id=");

            endpoint = id_list
                .iter()
                .fold(endpoint, |mut endpoint, id| {
                    endpoint.push_str(id);
                    endpoint.push(',');
                    endpoint
                });

            let endpoint = &endpoint[..endpoint.len() - 1];

            let mut reader = send_request(endpoint, &client_options)?;

            let result: Users =
                serde_json::from_reader(&mut reader)?;

            Ok(result)
        };

        // The limit per request is 100 users, so if more than 100 are needed, they are divided into chunks of 100 and requests are made in parallel
        let result = if ids.len() <= 100 {
            request_closure(ids)?
        } else {
            Users { users:
                ids
                .par_chunks(100)
                .map(request_closure)
                .fold(
                | | Vec::new(),
            |mut acc, b: Result<Users, Error>| {
                match b {
                    Ok(mut b) => {
                        acc.append(&mut b.users);
                    },
                    Err(err) => {
                        println!("{:?}", err);
                    }
                }
                acc
            },
            )
            .reduce(
                || Vec::new(),
                |mut acc: Vec<User>, mut b: Vec<User>| {
                    acc.append(&mut b);
                    acc
                },
            )
        }
        };

        Ok(result)
    }
}

impl User {
    // get a single user
    // equivalent to doing Users::by_names(..) and getting the first one
    pub fn by_name(name: &str, client_options: &ClientOptions) -> Result<Option<User>, Error> {
        let name_list = vec![name];
        let users = Users::by_names(&name_list[..], &client_options)?;

        Ok(users.users.into_iter().nth(0))
    }
}

#[derive(Debug, Fail)]
pub enum CacheError {
    #[fail(display = "Could not find user {}", 0)]
    UserNotFound(String),
}

/// A cache that stores id->user mappings in order to reduce api calls
pub struct UserIdCache {
    client_options: ClientOptions,
    cache: HashMap<String, User>,
}

impl UserIdCache {
    /// Create a new cache
    pub fn new(client_options: ClientOptions) -> Self {
        Self {
            client_options,
            cache: HashMap::new(),
        }
    }

    /// Input a bunch of id's into the cache.
    /// This will fetch any user that isn't already in the cache.
    /// Note that this will do batch api calls so this should be called with a series of user before using user(...) on all of them
    /// if api call limits are a concern
    pub fn batch_ids(&mut self, id_list: &[&str]) -> Result<(), Error> {
        let cache = &mut self.cache;
        let client_options = &self.client_options;

        let ids_to_fetch = id_list
            .into_iter()
            .filter_map(|id| {
                let id = *id;
                if cache.contains_key(id) {
                    None
                } else {
                    Some(id)
                }
            })
            .collect::<HashSet<&str>>()
            .into_iter()
            .collect::<Vec<&str>>();

        let users = Users::by_ids(&ids_to_fetch, client_options)?;

        users.users.into_iter().for_each(|user| {
            let _ = cache.insert(user._id.clone(), user);
        });

        Ok(())
    }

    /// Get a user, will fetch from api if needed.
    pub fn user(&mut self, id: &str) -> Result<User, Error> {
        if let Some(user) = self.cache.get(id) {
            return Ok(user.clone());
        }
        return {
            println!("Inserting {}", id);
            let users = Users::by_ids(&[id], &self.client_options)?.users;
            if users.len() > 0 {
                let _ = self.cache.insert(id.to_string(), users[0].clone());
                Ok(users[0].clone())
            } else {
                Err(CacheError::UserNotFound(id.to_string()).into())
            }
        };
    }
}
