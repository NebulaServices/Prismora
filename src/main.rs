use std::env;
use actix_web::{get, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, Duration};

#[derive(Serialize, Deserialize)]
struct MasqrError {
    error: String
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct MasqrResponse {
    assignedLicense: String,
    expires: i64
}

#[derive(Serialize, Deserialize, Debug)]
struct AllowedPsks {
    psks: Vec<String>
}

#[get("/")]
async fn index() -> impl Responder {
    "Hello, World!"
}

#[get("/newlicense")]
async fn newlicense(request: HttpRequest) -> impl Responder {
    let req_headers = request.headers();

    let psk_header = req_headers.get("PSK");

    let error = MasqrError {
        error: "Invalid PSK; Cannot assign licenses".to_owned()
    };

    match psk_header {
        Some(psk) => {
            let allowed_psks = "PSK";
            match env::var(allowed_psks) {
                Ok(val) => {
                    match serde_json::from_str::<Vec<String>>(&val) {
                        Ok(allowed_psks_parsed) => {
                            let psk = psk.to_str().unwrap().to_owned();

                            if allowed_psks_parsed.contains(&psk) {
                                let masqr_uuid = Uuid::new_v4().to_string();
                                let key = masqr_uuid[..6].to_owned();

                                let masqr_response = MasqrResponse {
                                    assignedLicense: key,
                                    expires: (Utc::now() + Duration::days(3)).timestamp_millis()
                                };

                                return HttpResponse::Ok().body(serde_json::to_string(&masqr_response).unwrap().to_owned());
                            } else {
                                return HttpResponse::InternalServerError().body(serde_json::to_string(&error).unwrap().to_owned());
                            } 
                        },
                        Err(e) => {
                            println!("Error parsing JSON: {}", e);
                            return HttpResponse::InternalServerError().body(serde_json::to_string(&error).unwrap().to_owned());
                        }
                    }
                },
                Err(e) => {
                    println!("Couldn't interpret {allowed_psks} environment variable: {e}");
                    return HttpResponse::InternalServerError().body(serde_json::to_string(&error).unwrap().to_owned());
                },
            }
        },
        None => {
            HttpResponse::InternalServerError().body(serde_json::to_string(&error).unwrap().to_owned())
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let allowed_psks = "PSK";
    match env::var(allowed_psks) {
        Ok(_) => (),
        Err(e) => println!("Couldn't interpret {allowed_psks} environment variable: {e}"),
    }

    HttpServer::new(|| App::new().service(index).service(newlicense))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}