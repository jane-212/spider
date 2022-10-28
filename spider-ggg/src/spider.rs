use spider_core::client;

use reqwest::{Client, StatusCode};

use spider_core::{
    error::{IError, IResult},
    image,
};

use serde::Deserialize;

use std::{
    fs::File,
    io::Read,
    time::Duration,
    thread,
};

use pbr::ProgressBar;

use nipper::Document;

use sqlx::{mysql::MySqlPoolOptions, Pool, MySql};

const CONFIG_PATH: &str = "spider-ggg.toml";

pub struct Spider {
    client: Client,
    config: Config,
    pool: Pool<MySql>,
}

#[derive(Deserialize, Debug)]
struct Config {
    site: Site,
    database: Database,
}

#[derive(Deserialize, Debug)]
struct Site {
    base_url: String,
    begin: u64,
    end: u64,
}

#[derive(Deserialize, Debug)]
struct Database {
    database_url: String,
    max_connection: u32,
}

impl Spider {
    pub async fn new() -> IResult<Self> {
        let mut config = String::new();

        let _ = File::open(CONFIG_PATH)
            .map_err(|_| IError::File(format!("can't open file {}", CONFIG_PATH)))?
            .read_to_string(&mut config);

        let config: Config = toml::from_str(&config)
            .map_err(|_| IError::Config(format!("a mistake found in {}", CONFIG_PATH)))?;

        let pool = MySqlPoolOptions::new()
        .max_connections(config.database.max_connection)
        .connect(&config.database.database_url)
        .await
        .map_err(|_| IError::Database("can't connect to database".into()))?;

        Ok(Self {
            client: client::new()?,
            config,
            pool,
        })
    }

    pub async fn run(self) -> IResult<()> {
        let begin = self.config.site.begin;
        let end = self.config.site.end;

        println!("--------------------------------------");

        for i in begin..end {
            let url = self.next(i);

            println!("{}", url);

            let page = self.get_page(&url).await?;

            let hrefs = self.parse_page(&page);

            let mut pb = ProgressBar::new(hrefs.len() as u64);

            pb.reset_start_time();

            for href in hrefs.into_iter() {
                let mut count = 1;

                loop {
                    let url = self.next_href(&href, count);
                    count += 1;

                    let href_page = self.get_href(&url).await?;

                    if let Some(href_page) = href_page {
                        if let Some(src) = self.parse_href(&href_page) {
                            image::insert_image(&self.pool, &src).await?;
                        }
                    } else {
                        break;
                    }

                    thread::sleep(Duration::from_secs(3));
                }

                pb.inc();
                thread::sleep(Duration::from_secs(3));
            }

            pb.finish();
            println!();
            println!("done");
            println!("--------------------------------------");
        }

        Ok(())
    }

    fn parse_href(&self, page: &str) -> Option<String> {
        let doc = Document::from(page);

        doc.select("#showpicnow")
            .first()
            .attr("src")
            .map(|src| src.to_string())
    }

    async fn get_href(&self, url: &str) -> IResult<Option<String>> {
        let res = reqwest::get(url)
            .await
            .map_err(|_| IError::Internet(format!("can't connect to the url: {}", url)))?;

        if res.status() != StatusCode::OK {
            return Ok(None);
        }

        Ok(Some(res.text().await.map_err(|_| {
            IError::Internet(format!("can't parse url: {}", url))
        })?))
    }

    fn next_href(&self, href: &str, count: usize) -> String {
        format!("{}_{}.html", href[..href.len() - 5].to_owned(), count)
    }

    fn parse_page(&self, page: &str) -> Vec<String> {
        let doc = Document::from(page);

        let mut hrefs = Vec::new();

        doc.select(".contlistw")
            .first()
            .select(".cl")
            .first()
            .select(".imgw")
            .iter()
            .for_each(|imgw| {
                if let Some(href) = imgw.attr("href") {
                    hrefs.push(href.to_string());
                }
            });

        hrefs
    }

    async fn get_page(&self, url: &str) -> IResult<String> {
        let html = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|_| IError::Internet(format!("can't connect to the url: {}", url)))?
            .text()
            .await
            .map_err(|_| IError::Internet(format!("can't parse url: {}", url)))?;

        Ok(html)
    }

    fn next(&self, index: u64) -> String {
        format!(
            "{}/meinv/index_{}.html",
            self.config.site.base_url,
            index.to_owned()
        )
    }
}
