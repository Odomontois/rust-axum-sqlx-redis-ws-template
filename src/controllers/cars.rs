
use crate::cache::CacheExt;
use crate::error::{AppError, AppJson};
use crate::models::car::{Car, CarList, CarQuery, NewCar};
use crate::repositories::car::HasCarRepo;
use crate::router::CARS_TAG;
use crate::services;
use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};

/// List all available Cars
///
/// Tries to all Cars from the database.
#[utoipa::path(
    get,
    path = "/list",
    responses((status = OK, body = [Car])),
    tag = CARS_TAG
)]
pub async fn list<S: HasCarRepo>(
    State(state): State<S>,
    Query(conditions): Query<CarQuery>,
) -> Result<AppJson<CarList>, AppError> {
    let cars = services::cars::search(state.car_repo(), &conditions).await?;
    Ok(AppJson(cars))
}

/// Search all cars
///
/// Tries to get list of cars by query from the database
#[utoipa::path(
    get,
    path = "/search",
    params(("name" = String, Query, description="Car Name")),
    responses((status = OK, body = [Car])),
    tag = CARS_TAG
)]
pub async fn search<S: HasCarRepo>(
    Query(params): Query<CarQuery>,
    State(state): State<S>,
) -> Result<AppJson<CarList>, AppError> {
    let cars = services::cars::search(state.car_repo(), &params).await?;
    Ok(AppJson(cars))
}

/// Get single Car by id
///
/// Tries to get single car by id from the database
#[utoipa::path(
    get,
    path = "/{car_id}",
    params(("car_id" = i32, Path, description="Car Id")),
    responses((status = OK, body = [Car])),
    tag = CARS_TAG
)]
pub async fn view<S: HasCarRepo>(
    Path(car_id): Path<i32>,
    State(state): State<S>,
    Extension(cache): CacheExt,
) -> Result<AppJson<Car>, AppError> {
    let car = services::cars::view(state.car_repo(), cache.clone(), car_id).await?;
    Ok(AppJson(car))
}

/// Create new Car
///
/// Tries to create a new Car in the database.
#[utoipa::path(
        post,
        path = "/create",
        tag = CARS_TAG,
        request_body(content=NewCar, content_type="application/json", description="New Car Information"),
        responses(
            (status = 201, description = "Car item created successfully", body = Car)
        )
)]
pub async fn create<S: HasCarRepo>(
    State(state): State<S>,
    Json(new_car): Json<NewCar>,
) -> Result<AppJson<Car>, AppError> {
    let car = services::cars::create(state.car_repo(), &new_car).await?;
    Ok(AppJson(car))
}

/// Update existing Car
///
/// Tries to update a Car in the database.
#[utoipa::path(
        post,
        path = "/update",
        tag = CARS_TAG,
        request_body(content=Car, content_type="application/json", description="Car To Update"),
        responses(
            (status = 200, description = "Car item updated successfully", body = Car)
        )
)]
pub async fn update<S: HasCarRepo>(
    State(state): State<S>,
    Json(car): Json<Car>,
) -> Result<AppJson<Car>, AppError> {
    let car = services::cars::update(state.car_repo(), &car).await?;
    Ok(AppJson(car))
}

/// Delete existing Car
///
/// Tries to delete a Car from the database.
#[utoipa::path(
        delete,
        path = "/delete/{car_id}",
        params(("car_id" = i32, Path, description="Car Id")),
        tag = CARS_TAG,
        responses(
            (status = 200, description = "Car item deleted successfully", body = String)
        )
)]
pub async fn delete<S: HasCarRepo>(
    Path(car_id): Path<i32>,
    State(state): State<S>,
) -> Result<(), AppError> {
    services::cars::delete(state.car_repo(), car_id).await?;
    Ok(())
}

// Example of end-to-end test with real database and repository
// 1. run `docker-compose -f compose-tests.yaml up -d` to start up the test db server
// 2. remove #[ignore] on the test method
#[cfg(test)]
mod tests {
    use crate::app::state::TestState;
    use crate::app::state::test_state;
    use crate::config::Config;
    use crate::controllers::cars;
    use crate::models::car::{CarList, NewCar};
    use crate::repositories::car::{CarRepository, CarRepositoryImpl};
    use crate::repositories::{clear_database, create_car_repository, run_migrations};
    use axum::http::Request;
    use axum::routing::get;
    use axum::{body::Body, http::StatusCode, Router};
    use once_cell::sync::Lazy;
    use tower::ServiceExt;

    static INIT: Lazy<()> = Lazy::new(|| {
        dotenv::from_filename(".env.test").ok();
        println!("Test environment loaded");
    });

    #[tokio::test]
    #[ignore]
    async fn list() {
        Lazy::force(&INIT);
        let config = Config::init();
        let _ = run_migrations(&config).await;
        let _ = clear_database(&config).await;
        let real_repo = create_car_repository(&config).await;

        // given
        let car = NewCar {
            name: "Tesla".to_string(),
            color: Some("Red".to_string()),
            year: Some(2020),
        };
        real_repo.create(&car).await.unwrap();

        // Create an Axum router with the mock repository as an extension
        let app = Router::new().route("/cars", get(cars::list::<TestState<CarRepositoryImpl>>));

        // Build a request to simulate a GET /cars
        let request = Request::builder()
            .uri("/cars?name=Tesla")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        // Use `oneshot` to send a single request through the router
        let service = tower::ServiceBuilder::new()
            .service(app)
            .with_state(test_state(real_repo));
        // when
        let response = service
            .oneshot(request)
            .await
            .expect("Failed to execute request");

        // then
        // Check the response status code
        assert_eq!(response.status(), StatusCode::OK);
        let max_body_size = 10 * 1024;
        let response_body = axum::body::to_bytes(response.into_body(), max_body_size)
            .await
            .expect("Failed to read body");
        let cars: CarList =
            serde_json::from_slice(&response_body).expect("Failed to deserialize response");
        assert_eq!(cars[0].name, "Tesla");
        assert_eq!(cars[0].color, Some("Red".to_string()));
        assert_eq!(cars[0].year, Some(2020));
    }
}
