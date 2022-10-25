mod spider;

use spider_core::error::IResult;

#[tokio::main]
async fn main() -> IResult<()> {
    spider_core::show_banner();

    let spider = spider::Spider::new().await?;
    spider.run().await
}
