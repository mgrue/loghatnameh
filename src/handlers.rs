use askama::Template;
use serde::Deserialize;
use axum::{
    http::{StatusCode},
    response::{Html, IntoResponse, Response},
    extract::{Form, State, Query}
};
use log::{debug};

#[derive(Debug, sqlx::Type)]
enum Lang {
    DE, FA
}

#[derive(Debug, sqlx::FromRow)]
struct Word {
    id: u32,
    value: String,
    lang: Lang
}

impl From<String> for Lang {
    fn from(value: String) -> Lang {
        let s = &*value;
        match s {
            "DE" => Lang::DE,
            "FA" => Lang::FA,
            &_ => Lang::DE
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct Variant {
    id: u32,
    name: String,
    value: String
}

struct Translation {
    word: Word,
    translations: Vec<Word>
}

#[derive(Deserialize, Debug)]
pub struct WordIdParam {
  id: u32
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<> {}

#[derive(Template)]
#[template(path = "results.html")]
struct ResultsTemplate<'a> {
    results: &'a Vec<Translation>
}

#[derive(Template)]
#[template(path = "no_results.html")]
struct NoResultsTemplate<> {
}

#[derive(Template)]
#[template(path = "word-details.html")]
struct WordDetailsTemplate<'a> {
    word: &'a Word,
    variants: &'a Vec<Variant>
}


#[derive(Deserialize)]
pub struct Search {
    query: String
}

#[derive(Clone)]
pub struct AppState {
    pub db_pool: sqlx::Pool<sqlx::MySql>
}

pub async fn root(State(state): State<AppState>) -> Response {
    log_qeuery(&state.db_pool, "PAGE_VIEW").await.unwrap();

    let template = IndexTemplate { };
    let html = template.render().unwrap();

    Html(html).into_response()
}

pub async fn search(State(state): State<AppState>, Form(payload): Form<Search>) -> Response {
    debug!("Query: {}", payload.query);

    log_qeuery(&state.db_pool, "SEARCH").await.unwrap();

    let query = sqlx::query_as!(Word, "SELECT * FROM word a where a.value like ?", payload.query);
    let words = query.fetch_all(&state.db_pool).await;

    let items = create_translations(words.unwrap(), &state.db_pool).await.unwrap();

    match items.len() {
        0 => empty_response(),
        _ => results_response(&items)
    }
}

pub async fn word_details(State(state): State<AppState>, word_id: Option<Query<WordIdParam>>) -> impl IntoResponse {
    log_qeuery(&state.db_pool, "WORD_VIEW").await.unwrap();
    
    let query = sqlx::query_as!(
        Word, 
        "select a.id, a.value, a.lang from word a where a.id = ?", 
        word_id.unwrap().id);
    let word = query.fetch_one(&state.db_pool).await.unwrap();

    let variant_query = sqlx::query_as!(
        Variant,
        "SELECT a.id, a.name, a.value FROM variant a WHERE a.fk_word_id = ?",
        word.id
    );

    let variants = variant_query.fetch_all(&state.db_pool).await;
    
    let template = WordDetailsTemplate { 
        word: &word,
        variants: &variants.unwrap()
    };
    let html = template.render().unwrap();

    (StatusCode::OK, Html(html).into_response())
}

fn empty_response() -> Response {
    let template = NoResultsTemplate {};
    let html = template.render().unwrap();

    Html(html).into_response()
}

fn results_response(items: &Vec<Translation>) -> Response {
    let template = ResultsTemplate { 
        results: &items 
    };
    let html = template.render().unwrap();

    Html(html).into_response()
}

async fn create_translations(words: Vec<Word>, db_pool: &sqlx::Pool<sqlx::MySql>) -> Result<Vec<Translation>, sqlx::Error> {
    let mut result: Vec<Translation> = vec![];
    
    for word in words {
        
        let translations = fetch_translations(&word, db_pool).await?;
        
        let translation = Translation {
            word: word,
            translations: translations
        };

        result.push(translation);
    }

    return Ok(result);
}

async fn fetch_translations(word: &Word, db_pool: &sqlx::Pool<sqlx::MySql>) -> Result<Vec<Word>, sqlx::Error> {
    let query = sqlx::query_as!(
        Word, 
        "select a.id, a.value, a.lang from word a left join translation b on a.id = b.fk_word_2_id where b.fk_word_1_id = ? or b.fk_word_2_id = ?", 
        word.id,
        word.id);
    let words = query.fetch_all(db_pool).await?;
    
    return Ok(words);
}

async fn log_qeuery(db_pool: &sqlx::Pool<sqlx::MySql>, query_type: &str) -> Result<(), sqlx::Error> {
    let sql = "INSERT INTO query_log(timestamp, query_type) VALUES(NOW(), ?);";

    sqlx::query(sql)
        .bind(query_type)
        .execute(db_pool)
        .await?;

    Ok(())
}