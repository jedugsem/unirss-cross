use crate::dir;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Imag {
    pub id: String,
    pub name: Option<String>,
}

pub async fn favicons(vec: Vec<(String, String)>) -> String {
    let mut r = String::from("error");
    for i in vec {
        let path = dir().join("cache").join(i.0.clone());
        let _ = std::fs::create_dir_all(path);
        let path = dir().join("cache").join(i.0.clone()).join("icon.ron");
        let mut icon = Imag {
            id: i.0.clone(),
            name: None,
        };
        if let Ok(res) = get_icon_url(i.1.clone()).await {
            icon.name = Some(res.1.clone());
            let path_img = dir().join("cache").join(i.0).join(res.1);

            println!("Img: {:?}", path_img);
            println!("Url: {:?}", res.0);
            match reqwest::get(res.0).await {
                Ok(dd) => {
                    if let Ok(bytes) = dd.bytes().await {
                        let _ = std::fs::write(path_img, &bytes);
                    }
                }
                Err(er) => {
                    r = format!("{:?}", er);
                }
            }
        }
        let mut file = File::create(path.clone()).unwrap();
        let _ =
            ron::Options::default().to_io_writer_pretty(&mut file, &icon, PrettyConfig::default());
    }
    r
}
use webpage::HTML;
pub async fn get_icon_url(url: String) -> Result<(String, String), String> {
    //let url2 = input.nth(1).unwrap();

    let mut url = url::Url::parse(&url).unwrap();
    url.path_segments_mut().unwrap().clear();
    url.set_query(None);
    let base_url = url.to_string();
    println!("base_url {}", base_url);
    let mut re = String::new();
    let mut logo_url: Option<String> = None;
    let hi = reqwest::get(&base_url).await;
    if let Ok(res) = hi {
        if let Ok(body) = res.text().await {
            let html = HTML::from_string(body.clone(), Some(base_url.clone())).unwrap();

            // println!("feed {:?}", info.html.opengraph.images);
            // println!("feed {:?}", info.html.schema_org);
            //println!("feed {:?}\n\n", info.html.meta);
            logo_url = if let Some(logo) = html.meta.get("og:logo") {
                Some(logo.to_string())
            } else if let Some(image) = html.meta.get("og:img") {
                Some(image.to_string())
            } else if let Some(msimage) = html.meta.get("msapplication-TileImage") {
                Some(msimage.to_string())
            } else if !html.opengraph.images.is_empty() {
                Some(html.opengraph.images[0].url.to_string())
            } else {
                get_img(&body).map(|icon| icon.to_string())
            };
        }
    } else if let Err(err) = hi {
        re = format!("{:?}", err);
    }

    if let Some(url) = &mut logo_url {
        if !url.starts_with("http") {
            *url = format!("{}{}", base_url.trim_end_matches("/"), url);
        }
        let final_url = url::Url::parse(url).unwrap();
        let name = final_url.path_segments().unwrap().next_back().unwrap();

        Ok((url.to_string(), name.to_string()))
    } else {
        Err(re)
    }
}
fn get_img(body: &str) -> Option<&str> {
    body.split("\n")
        .find(|x| x.trim_start().starts_with("<img src"))?
        .trim_start()
        .trim_start_matches("<img src=\"")
        .split("\"")
        .next()
}
