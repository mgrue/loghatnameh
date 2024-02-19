use askama::Template;
use serde::{Deserialize, Serialize};
use axum::{
    response::{Html, IntoResponse, Response},
    extract::{Form, State, Query}
};
//use dotenv::var;
use log::{debug};
use sqlx::{Error, Row};
use sqlx::mysql::MySqlQueryResult;

use crate::i18n;

#[derive(Debug, sqlx::FromRow)]
struct Word {
    id: u32,
    value: String,
    lang: String,
    transcript: Option<String>
}

#[derive(Debug, sqlx::FromRow)]
struct Variant {
    //id: u32,
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

#[derive(Deserialize, Debug)]
pub struct WordParam {
    query: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct AddWordParam {
    word1: Option<String>,
    word2: Option<String>,
    transcript: Option<String>,
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

#[derive(Template)]
#[template(path = "word-add.html")]
struct WordAddTemplate<'a> {
    value1: &'a Option<String>,
    value2: &'a Option<String>
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate<'a> {
    error: &'a sqlx::Error
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
    i18n::get_string("PRESENT_STEM");
    log_query(&state.db_pool, "PAGE_VIEW").await.unwrap();

    let template = IndexTemplate { };
    let html = template.render().unwrap();

    Html(html).into_response()
}

pub async fn search(State(state): State<AppState>, Form(payload): Form<Search>) -> Response {
    debug!("Query: {}", payload.query);

    log_query(&state.db_pool, "SEARCH").await.unwrap();

    let query = sqlx::query_as!(Word, "SELECT a.id, a.value, a.lang, a.transcript FROM word a where a.value like ?",
        format!("{}%", payload.query.trim()));
    let words = query.fetch_all(&state.db_pool).await;

    let items = create_translations(words.unwrap(), &state.db_pool).await;
    match items {
        Ok(items) => match items.len() {
            0 => empty_response(),
            _ => results_response(&items)
        }
        Err(e) => error_response(&e)
    }
}

pub async fn word_details(State(state): State<AppState>, word_id: Option<Query<WordIdParam>>) -> Response {
    log_query(&state.db_pool, "WORD_VIEW").await.unwrap();
    
    let query = sqlx::query_as!(
        Word, 
        "select a.id, a.value, a.lang, a.transcript from word a where a.id = ?",
        word_id.unwrap().id);
    let word_query_result = query.fetch_one(&state.db_pool).await;

    if word_query_result.is_err() {
        return error_response(&word_query_result.err().unwrap());
    }

    let word = word_query_result.unwrap();
    let variant_query = sqlx::query_as!(
        Variant,
        "SELECT a.name, a.value FROM variant a WHERE a.fk_word_id = ?",
        word.id
    );

    let query_result = variant_query.fetch_all(&state.db_pool).await;
    match query_result {
        Ok(variants) => word_details_response(&word, &variants),
        Err(error) => error_response(&error)
    }
}

pub async fn add_word_get(Query(query): Query<WordParam>) -> Response {
    let template = WordAddTemplate {
        value1: &query.query,
        value2: &None
    };
    let html = template.render().unwrap();

    Html(html).into_response()
}

pub async fn add_word_post(State(state): State<AppState>, Form(query): Form<AddWordParam>) -> Response {
    if query.word1.is_none() || query.word2.is_none() {
        return Html("<span class='error'>Two words must be provided</span>").into_response()
    }
    else {
        let transaction = state.db_pool.begin().await;
        let word1 = query.word1.unwrap();
        let word2 = query.word2.unwrap();

        match transaction {
            Ok(mut tx) => {
                let word1_insert = sqlx::query("INSERT INTO word(value, lang) VALUES(?, ?)")
                    .bind(word1)
                    .bind("DE")
                    .execute(&mut *tx)
                    .await;

                let word2_insert = sqlx::query("INSERT INTO word(value, transcript, lang) VALUES(?, ?, ?)")
                    .bind(word2)
                    .bind(query.transcript)
                    .bind("FA")
                    .execute(&mut *tx)
                    .await;

                if word1_insert.is_err() || word2_insert.is_err() {
                    tx.rollback().await.unwrap();
                    return error_response(&word1_insert.err().unwrap());
                }

                let word1_id = word1_insert.unwrap().last_insert_id();
                let word2_id = word2_insert.unwrap().last_insert_id();

                let translation_insert = sqlx::query("INSERT INTO translation(fk_word_1_id, fk_word_2_id) VALUES(?, ?)")
                    .bind(word1_id)
                    .bind(word2_id)
                    .execute(&mut *tx)
                    .await;

                if translation_insert.is_err() {
                    tx.rollback().await.unwrap();
                    return error_response(&translation_insert.err().unwrap());
                }

                tx.commit().await.unwrap();
            },
            Err(error) => return error_response(&error)
        }
    }

    Html("<span class='success'>DONE</span>").into_response()
}

fn word_details_response(word: &Word, variants: &Vec<Variant>) -> Response {
    let template = WordDetailsTemplate {
        word,
        variants
    };
    let html = template.render().unwrap();

    Html(html).into_response()
}

fn error_response(error: &sqlx::Error) -> Response {
    let template = ErrorTemplate {
        error
    };
    let html = template.render().unwrap();

    Html(html).into_response()
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
            word,
            translations
        };

        result.push(translation);
    }

    return Ok(result);
}

async fn fetch_translations(word: &Word, db_pool: &sqlx::Pool<sqlx::MySql>) -> Result<Vec<Word>, sqlx::Error> {
    let translations = sqlx::query(
        "select a.fk_word_1_id, a.fk_word_2_id from translation a where a.fk_word_1_id = ? or a.fk_word_2_id = ?")
        .bind(word.id)
        .bind(word.id)
        .fetch_all(db_pool)
        .await;

    let ids = match translations {
        Ok(rows) => {
            rows.iter().map(|e| {
                let id1:u32 = e.get(0);
                let id2:u32 = e.get(1);

                if id1 == word.id {
                    id2
                }
                else {
                    id1
                }
            }).collect::<Vec<u32>>()
        },
        Err(e) => return Err(e)
    };

    let query_str = format!("SELECT a.id, a.value, a.lang, a.transcript FROM word a where a.id in ({})",
                            ids.iter().map(|e| e.to_string()).collect::<Vec<String>>().join(","));
    let query:Result<Vec<Word>, Error> = sqlx::query_as(query_str.as_str()).fetch_all(db_pool).await;

    return match query {
        Ok(words) => Ok(words),
        Err(e) => Err(e)
    }
}

async fn log_query(db_pool: &sqlx::Pool<sqlx::MySql>, query_type: &str) -> Result<(), sqlx::Error> {
    let sql = "INSERT INTO query_log(timestamp, query_type) VALUES(NOW(), ?);";

    sqlx::query(sql)
        .bind(query_type)
        .execute(db_pool)
        .await?;

    Ok(())
}