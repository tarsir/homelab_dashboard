use actix_files::NamedFile;
use actix_web::{App, HttpRequest, HttpServer, Responder, Result, get, web};

mod containers;
mod services;

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

#[get("/api/services")]
async fn get_services_endpoint() -> Result<impl Responder> {
    Ok(web::Html::new(services::render_services_html()))
}

async fn dashboard() -> Result<NamedFile> {
    Ok(NamedFile::open("./static/index.html")?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "7001".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);

    println!("Starting server on {}", bind_addr);
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(dashboard))
            .service(hello)
            .service(get_services_endpoint)
    })
    .bind(bind_addr)?
    .run()
    .await
}
