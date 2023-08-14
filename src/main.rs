use utils::controllers::run;

mod connect;
mod entity;
mod migration;
mod share;
mod utils;

#[actix_web::main]
async fn main() {
    run().await.unwrap_or_else(|err| eprintln!("{}", err));
}
