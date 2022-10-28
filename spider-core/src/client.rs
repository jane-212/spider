use std::time;

use reqwest::{Client, ClientBuilder};

use super::error::{IError, IResult};

pub fn new() -> IResult<Client> {
    ClientBuilder::new()
            .timeout(time::Duration::from_secs(5))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.0.0.0 Safari/537.36 Edg/106.0.1370.52")
            .build()
            .map_err(|_| IError::Client("build Client failed".into()))
}
