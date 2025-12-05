use crate::AppData;
use crate::errors::MyError;
use crate::security::{check_password, generate_token, hash_password, validate_token};
use crate::types::{LoginRequest, RegisterRequest, RegisterResponse};
use actix_web::web::{Data, Json, Path, Query};
use actix_web::{HttpResponse, Responder, get, post};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::Deserialize;
use tokio_postgres::types::ToSql;
use uuid::Uuid;

#[derive(Deserialize)]
struct SearchParams {
    f: Option<String>,
    s: Option<String>,
}

#[get("/search")]
async fn search(
    args: Query<SearchParams>,
    auth: BearerAuth,
    app_data: Data<AppData>,
) -> actix_web::Result<impl Responder> {
    validate_token(&auth)?;
    if args.f.is_none() && args.s.is_none() {
        return Ok(HttpResponse::BadRequest().body("Error!!! No filters provided!!!"));
    }

    let mut query: String = "
        SELECT u.id,
               u.first_name,
               u.second_name,
               u.is_male,
               u.birthdate,
               u.biography,
               u.city
        FROM users u
        WHERE
        "
    .into();

    let mut counter = 1;
    let mut params: Vec<String> = Vec::new();
    if args.f.is_some() {
        query.push_str(format!(" u.first_name LIKE ${}", counter).as_str());
        params.push(format!("{}%", args.f.clone().unwrap()));
        counter += 1;
    }
    if args.s.is_some() {
        if counter > 1 {
            query.push_str(" AND");
        }
        query.push_str(format!(" u.second_name LIKE ${}", counter).as_str());
        params.push(format!("{}%", args.s.clone().unwrap()));
    }
    query.push_str(" ORDER BY u.id");
    let connection = app_data.pool.get().await.map_err(MyError::PoolError)?;
    let rows = connection
        .query(
            &query,
            params
                .iter()
                .map(|p| p as &(dyn ToSql + Sync))
                .collect::<Vec<&(dyn ToSql + Sync)>>()
                .as_slice(),
        )
        .await
        .map_err(MyError::TokioPostgresError)?;
    let result: Vec<Result<RegisterResponse, tokio_postgres::Error>> = rows
        .iter()
        .map(|row| {
            Ok(RegisterResponse {
                id: row.try_get(0)?,
                first_name: row.try_get(1)?,
                second_name: row.try_get(2)?,
                is_male: row.try_get(3)?,
                birthdate: row.try_get(4)?,
                biography: row.try_get(5)?,
                city: row.try_get(6)?,
            })
        })
        .collect();
    if result.iter().any(Result::is_err) {
        Ok(HttpResponse::InternalServerError().body("Error!!! Cannot get result from SQL!!!"))
    } else {
        let body: Vec<RegisterResponse> = result.into_iter().map(Result::unwrap).collect();
        Ok(HttpResponse::Ok().json(body))
    }
}

#[get("/get/{id}")]
async fn get(
    id: Path<Uuid>,
    auth: BearerAuth,
    app_data: Data<AppData>,
) -> actix_web::Result<impl Responder> {
    validate_token(&auth)?;
    let uuid: Uuid = id.into_inner();
    let connection = app_data.pool.get().await.map_err(MyError::PoolError)?;
    let sql = "
        SELECT u.first_name,
               u.second_name,
               u.is_male,
               u.birthdate,
               u.biography,
               u.city
        FROM users u
        WHERE u.id = $1
        ";
    let row = connection
        .query_one(sql, &[&uuid])
        .await
        .map_err(MyError::TokioPostgresError)?;
    let result = RegisterResponse {
        id: uuid,
        first_name: row.try_get(0).map_err(MyError::TokioPostgresError)?,
        second_name: row.try_get(1).map_err(MyError::TokioPostgresError)?,
        is_male: row.try_get(2).map_err(MyError::TokioPostgresError)?,
        birthdate: row.try_get(3).map_err(MyError::TokioPostgresError)?,
        biography: row.try_get(4).map_err(MyError::TokioPostgresError)?,
        city: row.try_get(5).map_err(MyError::TokioPostgresError)?,
    };
    Ok(HttpResponse::Ok().json(result))
}

#[post("/login")]
async fn login(
    request: Json<LoginRequest>,
    app_data: Data<AppData>,
) -> actix_web::Result<impl Responder> {
    let connection = app_data.pool.get().await.map_err(MyError::PoolError)?;
    let row = connection
        .query_one(
            "SELECT u.id, u.password_hash FROM users u WHERE u.id = $1",
            &[&request.id],
        )
        .await
        .map_err(MyError::TokioPostgresError)?;
    let hash_str: String = row.try_get(1).map_err(MyError::TokioPostgresError)?;
    check_password(&request.password, &hash_str)?;
    let token = generate_token(request.id)?;
    Ok(HttpResponse::Ok().json(token))
}

#[post("/register")]
async fn register(
    request: Json<RegisterRequest>,
    app_data: Data<AppData>,
) -> actix_web::Result<impl Responder> {
    let password_hash = hash_password(&request.password)?;
    let id = Uuid::new_v4();
    let connection = app_data.pool.get().await.map_err(MyError::PoolError)?;
    let sql = "
        INSERT INTO users (
            id,
            password_hash,
            first_name,
            second_name,
            is_male,
            birthdate,
            biography,
            city
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING
            id,
            first_name,
            second_name,
            is_male,
            birthdate,
            biography,
            city";
    let row = connection
        .query_one(
            sql,
            &[
                &id,
                &password_hash,
                &request.first_name,
                &request.second_name,
                &request.is_male,
                &request.birthdate,
                &request.biography,
                &request.city,
            ],
        )
        .await
        .map_err(MyError::TokioPostgresError)?;
    let response = RegisterResponse {
        id: row.try_get(0).map_err(MyError::TokioPostgresError)?,
        first_name: row.try_get(1).map_err(MyError::TokioPostgresError)?,
        second_name: row.try_get(2).map_err(MyError::TokioPostgresError)?,
        is_male: row.try_get(3).map_err(MyError::TokioPostgresError)?,
        birthdate: row.try_get(4).map_err(MyError::TokioPostgresError)?,
        biography: row.try_get(5).map_err(MyError::TokioPostgresError)?,
        city: row.try_get(6).map_err(MyError::TokioPostgresError)?,
    };
    Ok(HttpResponse::Ok().json(response))
}
