use actix_cors::Cors;
use actix_web::{
    delete, get, middleware::Logger, post, put, web, App, HttpResponse, HttpServer, Responder,
    Result,
};
use chrono::Duration;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use serde::Deserialize;
use serde::Serialize;
use std::clone::Clone;
use std::collections::HashMap;
use std::ops::Add;
use std::str::FromStr;
use std::sync::Mutex;
use strum_macros::EnumString;

#[derive(EnumString, Clone, Serialize)]
enum ComingType {
    Gammeln,
    Connecten,
    Fokus,
}
#[derive(Clone, Serialize)]
struct Coming {
    edited: DateTime<Utc>,
    coming_type: ComingType,
    when: DateTime<Utc>,
}

struct AppState {
    presence: Mutex<HashMap<String, DateTime<Utc>>>,
    coming: Mutex<HashMap<String, Coming>>,
}

#[derive(Serialize)]
pub struct PresenceResponse {
    pub users: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct ComingResponse {
    pub users: HashMap<String, Coming>,
}

#[derive(Serialize)]
pub struct Response {
    pub message: String,
}

#[derive(Deserialize)]
struct Presence {
    nickname: String,
}

#[derive(Deserialize)]
struct ComingRequest {
    nickname: String,
    coming_type: String,
    when: String,
}

fn retention_coming(data: web::Data<AppState>) {
    let mut coming = data.coming.lock().unwrap();
    for (key, value) in coming.clone() {
        if (value.edited + Duration::hours(6) < Utc::now()) {
            coming.remove(&key);
        }
    }
}

fn retention_presence(data: web::Data<AppState>) {
    let mut presence = data.presence.lock().unwrap();
    for (key, value) in presence.clone() {
        if (value + Duration::hours(6) < Utc::now()) {
            presence.remove(&key);
        }
    }
}

#[get("/health")]
async fn healthcheck() -> impl Responder {
    let response = Response {
        message: "Everything is working fine".to_string(),
    };
    HttpResponse::Ok().json(response)
}

#[get("/presence")]
async fn get_presence(data: web::Data<AppState>) -> impl Responder {
    retention_presence(data.clone());
    let mut presence = data.presence.lock().unwrap();
    let mut new_map: HashMap<String, String> = HashMap::new();
    for (key, value) in presence.clone() {
        new_map.insert(key, value.to_rfc3339());
    }
    let response = PresenceResponse {
        users: new_map.clone(),
    };
    HttpResponse::Ok().json(response)
}

#[put("/presence")]
async fn put_presence(info: web::Json<Presence>, data: web::Data<AppState>) -> impl Responder {
    let mut presence = data.presence.lock().unwrap();
    presence.insert(info.nickname.clone(), Utc::now());

    HttpResponse::Ok().finish()
}

#[delete("/presence")]
async fn delete_presence(info: web::Json<Presence>, data: web::Data<AppState>) -> impl Responder {
    let mut presence = data.presence.lock().unwrap();
    presence.remove(info.nickname.as_str());
    HttpResponse::Ok().finish()
}

#[get("/coming")]
async fn get_coming(data: web::Data<AppState>) -> impl Responder {
    retention_coming(data.clone());
    let mut coming = data.coming.lock().unwrap();
    let response = ComingResponse {
        users: coming.clone(),
    };
    HttpResponse::Ok().json(response)
}

#[put("/coming")]
async fn put_coming(info: web::Json<ComingRequest>, data: web::Data<AppState>) -> impl Responder {
    let mut coming = data.coming.lock().unwrap();
    println!("{:?}", info.when);
    let naive_datetime =
        NaiveDateTime::parse_from_str(info.when.as_str(), "%d.%m.%Y %H:%M:%S").unwrap();
    println!("{:?}", naive_datetime);
    let mut coming_data = Coming {
        when: TimeZone::from_utc_datetime(&Utc, &naive_datetime),
        coming_type: ComingType::from_str(info.coming_type.as_str()).unwrap(),
        edited: Utc::now(),
    };
    coming.insert(info.nickname.clone(), coming_data);
    HttpResponse::Ok().finish()
}

#[delete("/coming")]
async fn delete_coming(info: web::Json<Presence>, data: web::Data<AppState>) -> impl Responder {
    let mut coming = data.coming.lock().unwrap();
    coming.remove(info.nickname.as_str());
    HttpResponse::Ok().finish()
}

async fn not_found() -> Result<HttpResponse> {
    let response = Response {
        message: "Resource not found".to_string(),
    };
    Ok(HttpResponse::NotFound().json(response))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let appstate = web::Data::new(AppState {
        presence: Mutex::new(HashMap::new()),
        coming: Mutex::new(HashMap::new()),
    });
    HttpServer::new(move || {
        let cors = Cors::default()
            .supports_credentials()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();

        App::new()
            .app_data(appstate.clone())
            .wrap(Logger::default())
            .wrap(cors)
            .service(healthcheck)
            .service(get_presence)
            .service(put_presence)
            .service(delete_presence)
            .service(get_coming)
            .service(put_coming)
            .service(delete_coming)
            .default_service(web::route().to(not_found))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
