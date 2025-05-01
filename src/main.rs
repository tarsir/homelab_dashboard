use actix_files::NamedFile;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, Result};
mod containers;

#[get("/api/containers")]
async fn hello() -> Result<impl Responder> {
    Ok(web::Html::new(
        containers::get_container_list()
            .iter()
            .flat_map(|c| c.to_html_tr().chars().collect::<Vec<char>>())
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
