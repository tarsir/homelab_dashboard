use std::str::FromStr;

use actix_files::NamedFile;
use actix_web::{
    App, HttpRequest, HttpServer, Responder, Result, dev::ConnectionInfo, get, http::Uri, web,
};
mod containers;

#[get("/api/containers")]
async fn hello(req: HttpRequest) -> Result<impl Responder> {
    let host = req.full_url();
    let host = host.host_str().unwrap_or_default();
    Ok(web::Html::new(
        containers::get_container_list()
            .iter()
            .map(|c| c.to_html_card(host))
            .collect::<String>(),
    ))
}

async fn dashboard() -> Result<NamedFile> {
    Ok(NamedFile::open("./static/index.html")?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(dashboard))
            .service(hello)
    })
    .bind(("127.0.0.1", 7001))?
    .run()
    .await
}
