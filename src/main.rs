use axum::{
    extract::{Path, State},
    // routing::{get, post, put, delete},
    routing::get,
    Router, Json,
};
use cuid::cuid2_slug;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Character {
    name: String,
    id: String,
    abilities: Vec<String>,
    bio: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CharacterRequest {
    name: String,
    abilities: Vec<String>,
    bio: String,
}

type SharedState = Arc<Mutex<Vec<Character>>>;

#[tokio::main]
async fn main() {
    // Initialize shared state
    let characters: SharedState = Arc::new(Mutex::new(vec![
        Character {
            name: "Harry".to_string(),
            id: cuid2_slug(),
            abilities: vec!["Parcel Tongue".to_string()],
            bio: "Orphaned by Voldemort".to_string(),
        },
    ]));

    // build our application with routes
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/char", get(get_all_chars).post(create_char))
        .route(
            "/char/{id}",
            get(get_char_by_id)
                .put(update_char_by_id)
                .delete(delete_char_by_id),
        )
        .with_state(characters);

    println!("Server running on http://0.0.0.0:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_all_chars(
    State(state): State<SharedState>,
) -> Json<Vec<Character>> {
    let characters = state.lock().await;
    Json(characters.clone())
}

async fn create_char(
    State(state): State<SharedState>,
    Json(char_request): Json<CharacterRequest>,
) -> Json<Character> {
    let new_char = Character {
        name: char_request.name,
        id: cuid2_slug(),
        abilities: char_request.abilities,
        bio: char_request.bio,
    };

    let mut characters = state.lock().await;
    characters.push(new_char.clone());
    Json(new_char)
}

async fn get_char_by_id(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<Character>, String> {
    let characters = state.lock().await;
    characters
        .iter()
        .find(|c| c.id == id)
        .map(|c| Json(c.clone()))
        .ok_or_else(|| "Character not found".to_string())
}

async fn update_char_by_id(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(char_request): Json<CharacterRequest>,
) -> Result<Json<Character>, String> {
    let mut characters = state.lock().await;

    if let Some(character) = characters.iter_mut().find(|c| c.id == id) {
        character.name = char_request.name;
        character.abilities = char_request.abilities;
        character.bio = char_request.bio;
        Ok(Json(character.clone()))
    } else {
        Err("Character not found".to_string())
    }
}

async fn delete_char_by_id(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<(), String> {
    let mut characters = state.lock().await;
    let initial_len = characters.len();
    characters.retain(|c| c.id != id);

    if characters.len() < initial_len {
        Ok(())
    } else {
        Err("Character not found".to_string())
    }
}
