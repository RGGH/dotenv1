#![allow(unused)]
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use log::{info, error}; // Add logging

#[derive(Clone)]
struct AppState {
    db_pool: PgPool,
    cache: Arc<Mutex<HashMap<String, String>>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: i32,
    name: String,
    email: String,
}

async fn get_user(data: web::Data<AppState>, user_id: web::Path<i32>) -> impl Responder {
    let user_id = user_id.into_inner();
    let cache_key = user_id.to_string();

    // Lock the cache for reading
    let cache = data.cache.lock().unwrap();

    // Check if the user is in the cache
    if let Some(user_data) = cache.get(&cache_key) {
        info!("Cache hit for user ID {}", user_id);
        return HttpResponse::Ok().body(user_data.clone());
    }

    info!("Cache miss for user ID {}", user_id);

    // If not in cache, fetch from the database
    let pool = &data.db_pool;
    match sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
        .fetch_one(pool)
        .await
    {
        Ok(user) => {
            // Serialize the user data
            let user_data = match serde_json::to_string(&user) {
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to serialize user data: {:?}", e);
                    return HttpResponse::InternalServerError().body("Internal server error");
                }
            };

            // Store the user data in the cache
            drop(cache); // Unlock the cache before updating it
            let mut cache = data.cache.lock().unwrap();
            cache.insert(cache_key, user_data.clone());

            info!("Stored user ID {} in cache", user_id);

            HttpResponse::Ok().body(user_data)
        }
        Err(sqlx::Error::RowNotFound) => {
            info!("User ID {} not found in database", user_id);
            HttpResponse::NotFound().body(format!("User with ID {} not found", user_id))
        }
        Err(e) => {
            error!("Database query failed: {:?}", e);
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::init(); // Initialize the logger
    dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let address = format!("0.0.0.0:{}", port);

    let database_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
    println!("{:?}", database_url);

    let db_pool = PgPool::connect(&database_url)
        .await
        .map_err(|e| {
            eprintln!("Failed to connect to the database: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, "Database connection error")
        })?;
    
    let cache = Arc::new(Mutex::new(HashMap::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                db_pool: db_pool.clone(),
                cache: cache.clone(),
            }))
            .route("/user/{id}", web::get().to(get_user))
    })
    .bind(&address)?
    .run()
    .await
}

