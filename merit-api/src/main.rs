#![feature(rustc_private)]

#[macro_use]
extern crate actix_web;

mod badge_routes;
mod services;
mod utils;

use actix_web::{
  dev,
  http::{header, StatusCode},
  middleware, web, App, HttpResponse, HttpServer, Result,
};
use dotenv::dotenv;
use env_logger::Env;
use listenfd::ListenFd;
use merit::*;
use std::{env, io};

#[get("/")]
fn index() -> Result<HttpResponse> {
  Ok(
    HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
      .header(header::LOCATION, "https://github.com/msuntharesan/merit")
      .finish(),
  )
}

#[get("/favicon.ico")]
fn favicon() -> Result<HttpResponse> {
  Ok(
    HttpResponse::Ok()
      .content_type("image/x-icon")
      .body(dev::Body::from_slice(include_bytes!(
        "../static/favicon.ico"
      ))),
  )
}

fn default_404() -> Result<HttpResponse> {
  let mut badge = Badge::new("Error");
  badge.text("404").color("grey");

  Ok(
    HttpResponse::NotFound()
      .content_type("image/svg+xml")
      .body(badge.to_string()),
  )
}

fn main() -> io::Result<()> {
  dotenv().ok();
  let env = Env::new().filter("LOG_LEVEL");
  env_logger::init_from_env(env);

  let mut listenfd = ListenFd::from_env();

  let sys = actix_rt::System::new("badge");

  let mut server = HttpServer::new(move || {
    App::new()
      .wrap(middleware::Logger::default())
      .wrap(middleware::NormalizePath)
      .wrap(
        middleware::DefaultHeaders::new()
          .header("Cache-Control", format!("public, max-age={}", 60 * 24)),
      )
      .default_service(web::route().to(default_404))
      .service(index)
      .service(favicon)
      .configure(badge_routes::config)
      .configure(services::crates_io::config)
      .configure(services::github::config)
      .configure(services::npm::config)
  });

  server = if let Some(l) = listenfd.take_tcp_listener(0)? {
    server.listen(l).unwrap()
  } else {
    let port = env::var("PORT").unwrap_or("3000".into());
    let addr = format!("0.0.0.0:{}", port);
    println!("Listening on {}", addr);
    server.bind(addr)?
  };
  server.start();
  sys.run()
}
