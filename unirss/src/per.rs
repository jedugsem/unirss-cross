use crate::settings::PSettings;
use crate::{
    dir,
    git::{self, GitState},
    OnlineFeeds, Progress,
};

use ron::{de::from_reader, ser::PrettyConfig};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::BufReader;
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Imag {
    pub id: String,
    pub name: Option<String>,
}
pub async fn refresh(
    git_state: Option<GitState>,
) -> (
    Option<Vec<(feed_rs::model::Feed, Imag)>>,
    Option<Vec<(String, String)>>,
) {
    if let Some(git_state) = git_state {
        let _ = git::pull(git_state).await;
    }

    let online_feeds = read_online_feeds().await;
    let client = reqwest::Client::new();
    for i in online_feeds.urls {
        let bytes = client.get(i.1).send().await.unwrap().bytes().await.unwrap();
        let mut path = dir().join("feeds").join(i.0);
        path.set_extension("xml");

        fs::write(path, bytes).expect("Oh no");
    }

    read_feeds()
}
pub fn read_feeds() -> (
    Option<Vec<(feed_rs::model::Feed, Imag)>>,
    Option<Vec<(String, String)>>,
) {
    let mut mods = vec![];
    let mut missing = vec![];
    if let Ok(modules) = dir().join("git").join("xmls").read_dir() {
        for modul in modules {
            let modul = modul.unwrap();
            //println!("{:?}", modul);
            if let Some(osstr) = modul.path().extension() {
                if osstr.to_str() == Some("xml") {
                    let file = std::fs::File::open(modul.path()).unwrap();
                    let feed = feed_rs::parser::parse(BufReader::new(file)).unwrap();
                    let title = feed.title.clone();
                    let title = title.unwrap().content.clone();
                    let icon_path = dir().join("cache").join(title.clone()).join("icon.ron");
                    let mut imag = Imag::default();
                    println!("{:?}", icon_path);
                    if icon_path.exists() {
                        let file = std::fs::File::open(icon_path);
                        if let Ok(reader) = file {
                            if let Ok(icon) = from_reader(reader) {
                                imag = icon;
                            }
                        }
                    } else if !feed.links.is_empty() {
                        missing.push((title, feed.links[0].href.clone()))
                    }
                    mods.push((
                        feed, //
                        imag,
                    ));
                }
            }
        }
    }

    if let Ok(modules) = dir().join("feeds").read_dir() {
        for modul in modules {
            let modul = modul.unwrap();
            //println!("{:?}", modul);
            if let Some(osstr) = modul.path().extension() {
                if osstr.to_str() == Some("xml") {
                    let file = std::fs::File::open(modul.path()).unwrap();
                    let feed = feed_rs::parser::parse(BufReader::new(file)).unwrap();
                    let title = feed.title.clone();
                    let title = title.unwrap().content.clone();
                    let mut imag = Imag::default();
                    let icon_path = dir().join("cache").join(title.clone()).join("icon.ron");
                    if icon_path.exists() {
                        let file = std::fs::File::open(icon_path);
                        if let Ok(reader) = file {
                            if let Ok(icon) = from_reader(reader) {
                                imag = icon;
                            }
                        }
                    } else if !feed.links.is_empty() {
                        missing.push((title, feed.links[0].href.clone()))
                    }

                    mods.push((
                        //
                        feed, imag,
                    ));
                }
            }
        }
    }
    (
        Some(mods),
        if missing.is_empty() {
            None
        } else {
            Some(missing)
        },
    )
}
pub async fn read_online_feeds() -> OnlineFeeds {
    let path = dir().join("git").join("feeds.ron");
    let file = std::fs::File::open(path);
    if let Ok(reader) = file {
        from_reader(reader).unwrap_or(OnlineFeeds::default())
    } else {
        OnlineFeeds::default()
    }
}
pub async fn write_online_feeds(urls: OnlineFeeds, git_state: GitState) {
    let path = dir().join("git").join("feeds.ron");
    let mut file = File::create(path.clone()).unwrap();
    let _ = ron::Options::default().to_io_writer_pretty(&mut file, &urls, PrettyConfig::default());
    _ = git::add(vec![path]).await;
    _ = git::commit("write progress").await;
    _ = git::push(git_state).await;
}
pub async fn read_progress() -> Progress {
    let path = dir().join("git").join("progress.ron");
    let file = std::fs::File::open(path);
    if let Ok(reader) = file {
        from_reader(reader).unwrap_or(Progress::default())
    } else {
        Progress::default()
    }
}
pub async fn write_progress(urls: Progress, git_state: GitState) {
    let path = dir().join("git").join("progress.ron");
    let mut file = File::create(path.clone()).unwrap();
    let _ = ron::Options::default().to_io_writer_pretty(&mut file, &urls, PrettyConfig::default());
    _ = git::add(vec![path]).await;
    _ = git::commit("write progress").await;
    _ = git::push(git_state).await;
}
pub fn load_settings() -> Result<crate::settings::PSettings, String> {
    let path = dir();
    match std::fs::File::open(path.join("config.ron")) {
        Ok(file) => Ok(from_reader(file).unwrap_or(PSettings::default())),
        _ => Err("".to_string()),
    }
}

pub fn write_settings(settings: crate::settings::PSettings) {
    let path = dir();
    let mut file = File::create(path.join("config.ron")).unwrap();
    let _ =
        ron::Options::default().to_io_writer_pretty(&mut file, &settings, PrettyConfig::default());
}
