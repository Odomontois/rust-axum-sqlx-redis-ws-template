use crate::{
    app::{AppState, IsState},
    controllers::{
        cars::{self, list},
        parts, utils,
    },
    repositories::car::HasCarRepo,
};
use axum::Router;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

pub const CARS_TAG: &str = "Cars";
pub const PARTS_TAG: &str = "Parts";
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = CARS_TAG, description = "Cars management API"),
        (name = PARTS_TAG, description = "Parts management API")
    )
)]
struct ApiDoc;
pub fn router<S: HasCarRepo + Unpin>() -> Router<S> {
    let app: OpenApiRouter<S> = OpenApiRouter::new()
        .routes(routes!(utils::healthcheck))
        .nest("/cars", car_routes())
        .nest("/parts", part_routes());

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api", app)
        .split_for_parts();

    let router = router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()))
        .merge(Redoc::with_url("/redoc", api.clone()))
        // There is no need to create `RapiDoc::with_openapi` because the OpenApi is served
        // via SwaggerUi instead we only make rapidoc to point to the existing doc.
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        // Alternative to above
        // .merge(RapiDoc::with_openapi("/api-docs/openapi2.json", api).path("/rapidoc"))
        .merge(Scalar::with_url("/scalar", api));

    Router::new().nest("/", router)
}

fn car_routes<S: HasCarRepo>() -> OpenApiRouter<S> {
    OpenApiRouter::<S>::new()
        .routes(routes!(cars::list::<S>))
        .routes(routes!(cars::search::<S>))
        .routes(routes!(cars::create::<S>))
        .routes(routes!(cars::view::<S>))
        .routes(routes!(cars::update::<S>))
        .routes(routes!(cars::delete::<S>))
}

fn part_routes<S: IsState>() -> OpenApiRouter<S> {
    OpenApiRouter::new()
        .routes(routes!(parts::index))
        .routes(routes!(parts::search))
        .routes(routes!(parts::create))
        .routes(routes!(parts::view))
        .routes(routes!(parts::update))
        .routes(routes!(parts::delete))
}
