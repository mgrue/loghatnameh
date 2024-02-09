use askama::Template;
use serde::Deserialize;
use axum::{
    http::{StatusCode},
    response::{Html, IntoResponse, Response},
    extract::{Form, State}
};

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

struct Translation {
    word: Word,
    translations: Vec<Word>
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

#[derive(Deserialize)]
pub struct Search {
    query: String
}

#[derive(Clone)]
pub struct AppState {
    pub db_pool: sqlx::Pool<sqlx::MySql>
}

pub async fn root(State(state): State<AppState>) -> impl IntoResponse {
    log_qeuery(&state.db_pool, "PAGE_VIEW").await.unwrap();

    let hello = IndexTemplate { };
    let html = hello.render().unwrap();

    (StatusCode::OK, Html(html).into_response())
}

pub async fn search(State(state): State<AppState>, Form(payload): Form<Search>) -> impl IntoResponse {
    println!("Query: {}", payload.query);

    log_qeuery(&state.db_pool, "SEARCH").await.unwrap();

    let query = sqlx::query_as!(Word, "SELECT * FROM word a where a.value like ?", payload.query);
    let words = query.fetch_all(&state.db_pool).await;

    let items = create_translations(words.unwrap(), &state.db_pool).await.unwrap();
    
    if items.len() > 0 {
        let template = ResultsTemplate { 
            results: &items 
        };
        let html = template.render().unwrap();
    
        return (StatusCode::OK, Html(html).into_response())
    } else {
        let template = NoResultsTemplate {};
        let html = template.render().unwrap();

        return (StatusCode::OK, Html(html).into_response())
    }
}

fn empty_response() -> impl IntoResponse {
    let template = NoResultsTemplate {};
    let html = template.render().unwrap();

    (StatusCode::OK, Html(html).into_response())
}

fn results_response(items: &Vec<Translation>) -> impl IntoResponse {
    let template = ResultsTemplate { 
        results: &items 
    };
    let html = template.render().unwrap();

    (StatusCode::OK, Html(html).into_response())
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