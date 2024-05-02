use actix_web::{error::ErrorNotFound, get, guard::Trace, http::header, post, web::{self, get, resource, to, Redirect, ReqData}, App, HttpServer};
use actix_files as fs;
use tera::{Tera, Context};
use actix_web::{HttpRequest, HttpResponse, Responder};
use serde_derive::{Serialize, Deserialize};
use actix_service::into_service;
use bcrypt::{hash, DEFAULT_COST};
use rusqlite::{Connection, Result};


mod db;

#[derive(Clone, Debug)]
struct AppData {
    tmpl: Tera
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct FormData {
    username: String,
    password: String,
}

#[get("/index")]
async fn index(data: web::Data<AppData>, req: HttpRequest) -> impl Responder{ 
    
    let username = req.query_string().split("=").last().unwrap_or_default();

    let mut ctx = Context::new();
    ctx.insert("username", username);
    let rendered: String = data.tmpl.render("index.html", &ctx).unwrap();
    
    HttpResponse::Ok()
        .header(header::CONTENT_TYPE, "text/html")
        .body(rendered)
}

#[get("/register")]
async fn show_register_form(data: web::Data<AppData>, _req: HttpRequest) -> impl Responder {
    let rendered: String = data.tmpl.render("register.html", &Context::new()).unwrap();
    HttpResponse::Ok()
        .header(header::CONTENT_TYPE, "text/html")
        .body(rendered)
}

#[post("/register")] // Use web::post() to define a POST endpoint
async fn register_user(form: web::Form<FormData>) -> impl Responder {
    let username = &form.username;
    let password = &form.password;

    if let Err(_) = db::user_db::register_user(username, password) {
        let redirect_url = "/login";
        return HttpResponse::Found()
            .header(header::LOCATION, redirect_url)
            .finish();
    }

    let redirect_url = format!("/index?username={}", username);
    HttpResponse::Found()
        .header(header::LOCATION, redirect_url)
        .finish()
}

#[get("/login")]
async fn login_get(data: web::Data<AppData>, _req: HttpRequest) -> impl Responder {
    let rendered: String = data.tmpl.render("login.html", &Context::new()).unwrap();
    HttpResponse::Ok()
        .header(header::CONTENT_TYPE, "text/html")
        .body(rendered)
}


#[post("/login")] 
async fn login_post(form: web::Form<FormData>) -> impl Responder {

    let username = &form.username;
    let password = &form.password;

    match db::user_db::check_credentials(username, password) {
        Ok(true) => {

            let redirect_url = format!("/index?username={}", username);
            HttpResponse::Found()
                .header(header::LOCATION, redirect_url)
                .finish()
        }
        Ok(false) => {

            let redirect_url = "/login?error=invalid_credentials"; 
            HttpResponse::Found()
                .header(header::LOCATION, redirect_url)
                .finish()
        }
        Err(_) => {

            let redirect_url = "/login?error=internal_error"; // Пример страницы входа с ошибкой
            HttpResponse::Found()
                .header(header::LOCATION, redirect_url)
                .finish()
        }
    }
}


#[post("/add_word")] 
async fn add_word(json_data: web::Json<db::Word>) -> impl Responder {
    let word = json_data.word.clone();
    let language = json_data.language.clone();
    println!("{:?}, {:?}", word, language);
    match db::word_db::add_word(&word, &language) {
        Ok(_) => {
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            eprintln!("Ошибка при добавлении слова: {}", e);
            HttpResponse::InternalServerError().finish() 
        }
    }
}

#[get("/get_words")]
async fn get_words(data: web::Data<AppData>, req: HttpRequest) -> impl Responder {
    println!("Data = {:?}\n\n\n req = {:?}", data, req);
    match db::word_db::fetch_words_from_db() {
        Ok(words) => {  
            println!("{:?}", words);
            HttpResponse::Ok().json(words)
            
        },
        Err(e) => {
            eprintln!("Error fetching words: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(show_register_form)
        .service(register_user)
        .service(login_get)
        .service(login_post)
        .service(add_word)
        .service(get_words)
        .service(index);
        
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(err) = db::user_db::create_table() {
        eprintln!("Failed to create users table: {:?}", err);
    }

    if let Err(err) = db::word_db::create_table() {
        eprintln!("Failed to create words table: {:?}", err);
    }

    let tera =
        Tera::new(
            concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")
        ).unwrap();
    
        HttpServer::new(move || {
            App::new()
                .data(AppData{ tmpl: tera.clone()})
                
                 .configure(configure_routes)
        })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
