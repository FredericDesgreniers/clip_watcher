extern crate reqwest;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate twitch;

mod config;
mod file_util;

use config::Config;
use file_util::*;
use std::fs::File;
use std::io::Write;
use twitch::clips::Clip;
use twitch::clips::ClipToCreatorMap;
use twitch::ClientOptions;
use twitch::Twitch;

fn main() {
    let mut config_file = File::open("config.toml").expect("No configuration file present");
    let config_text = config_file
        .read_as_string()
        .expect("Could not read config file");

    let config = toml::from_str::<Config>(&config_text).expect("Error parsing config file");

    let client_options =
        twitch::ClientOptions::new(config.client_id.clone(), config.bearer.clone());

    clip_watcher(&client_options, &config);
}

fn clip_watcher(client_options: &ClientOptions, config: &Config) {
    let mut twitch = Twitch::new(client_options.clone());

    let user = twitch::user::User::by_name(&config.channel, &client_options)
        .unwrap()
        .expect("User");

    let mut user_id_cache = twitch.create_user_id_cache();

    let clips_options = twitch::clips::ClipsOptions {
        per_page: 100,
        after: twitch::clips::Pagination { cursor: None },
    };

    println!("Getting user clips...");

    let all_clips: Vec<Clip> = twitch
        .get_clips_by_broardcaster(&user._id, clips_options)
        .expect("All clips")
        .paginated_clips
        .into_iter()
        .fold(Vec::new(), |mut result, mut this| {
            result.append(&mut this.data);
            result
        }).into_iter().filter(|clip| clip.created_at.starts_with("2018-07")).collect();

    {
        let mut output = File::create("clips.csv").expect("Could not create clips file");

        writeln!(&output, "date, creator_id, clip_id, title, url, views").unwrap();

        all_clips.iter().for_each(|clip| {
            writeln!(
                &mut output,
                "{date}, {creator}, {clip},\"{title}\", {url}, {views}",
                creator = clip.creator_id,
                clip = clip.id,
                views = clip.view_count,
                date = clip.created_at,
                title = clip.title,
                url = clip.url
            ).unwrap();
        });
    }

    let ids = all_clips
        .iter()
        .map(|clip| &clip.creator_id as &str)
        .collect::<Vec<&str>>();

    user_id_cache.batch_ids(&ids).unwrap();
    println!("Mapping id -> username...");

    let creator_map = all_clips.get_creator_map();

    println!("Outputting top creators...");

    let mut creator_list: Vec<(&String, &(u32, u32))> = creator_map.iter().collect();

    creator_list.sort_by(|a, b| (b.1).1.cmp(&(a.1).1));

    let mut file = File::create("user.csv").expect("Could not create user file");

    writeln!(&mut file, "name, creator_id, clip_count, total_views").unwrap();

    for creator in creator_list {
        let name = if let Ok(user) = user_id_cache.user(&creator.0) {
            user.display_name
        } else {
            "NO NAME".to_string()
        };

        writeln!(
            &mut file,
            "{name}, {creator_id}, {clip_count}, {total_views}",
            name = name,
            creator_id = creator.0,
            clip_count = (creator.1).0,
            total_views = (creator.1).1
        ).unwrap();
    }
}
