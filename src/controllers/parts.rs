use crate::cache::CacheExt;
use crate::error::{AppError, AppJson};
use crate::models::part::{NewPart, Part, PartList, PartQuery};
use crate::repositories::part::HasPartRepo;
use crate::router::PARTS_TAG;
use crate::services;
use axum::extract::{Path, Query, State};
use axum::{extract::Extension, Json};

/// List all available Parts
///
/// Tries to all Parts from the database.
#[utoipa::path(
    get,
    path = "/list",
    responses((status = OK, body = [Part])),
    tag = PARTS_TAG
)]
pub async fn index<S: HasPartRepo>(
    Query(conditions): Query<PartQuery>,
    State(state): State<S>,
) -> Result<AppJson<PartList>, AppError> {
    let parts = services::parts::search(state.part_repo(), &conditions).await?;
    Ok(AppJson(parts))
}

/// Create new Part
///
/// Tries to create a new Part in the database.
#[utoipa::path(
        post,
        path = "/create",
        tag = PARTS_TAG,
        request_body(content=NewPart, content_type="application/json", description="New Part Information"),
        responses(
            (status = 201, description = "Part item created successfully", body = Part)
        )
)]
pub async fn create<S: HasPartRepo>(
    State(state): State<S>,
    Json(new_part): Json<NewPart>,
) -> Result<AppJson<Part>, AppError> {
    let part = services::parts::create(state.part_repo(), &new_part).await?;
    Ok(AppJson(part))
}

/// Get single Part by id
///
/// Tries to get single part by id from the database
#[utoipa::path(
    get,
    path = "/{part_id}",
    params(("part_id" = i32, Path, description="Part Id")),
    responses((status = OK, body = [Part])),
    tag = PARTS_TAG
)]
pub async fn view<S: HasPartRepo>(
    Path(part_id): Path<i32>,
    State(state): State<S>,
    Extension(cache): CacheExt,
) -> Result<AppJson<Part>, AppError> {
    let part = services::parts::view(state.part_repo(), cache.clone(), part_id).await?;
    Ok(AppJson(part))
}

/// Search all parts
///
/// Tries to get list of parts by query from the database
#[utoipa::path(
    get,
    path = "/search",
    params(("name" = String, Query, description="Part Name")),
    responses((status = OK, body = [Part])),
    tag = PARTS_TAG
)]
pub async fn search<S: HasPartRepo>(
    Query(params): Query<PartQuery>,
    State(state): State<S>,
) -> Result<AppJson<PartList>, AppError> {
    let parts = services::parts::search(state.part_repo(), &params).await?;
    Ok(AppJson(parts))
}

/// Update existing Part
///
/// Tries to update a Part in the database.
#[utoipa::path(
        post,
        path = "/update",
        tag = PARTS_TAG,
        request_body(content=Part, content_type="application/json", description="Part To Update"),
        responses(
            (status = 200, description = "Part item updated successfully", body = Part)
        )
)]
pub async fn update<S: HasPartRepo>(
    State(state): State<S>,
    Json(part): Json<Part>,
) -> Result<AppJson<Part>, AppError> {
    let part = services::parts::update(state.part_repo(), &part).await?;
    Ok(AppJson(part))
}

/// Delete existing Part
///
/// Tries to delete a Part from the database.
#[utoipa::path(
        delete,
        path = "/delete/{part_id}",
        params(("part_id" = i32, Path, description="Part Id")),
        tag = PARTS_TAG,
        responses(
            (status = 200, description = "Part item deleted successfully", body = String)
        )
)]
pub async fn delete<S: HasPartRepo>(
    Path(part_id): Path<i32>,
    State(state): State<S>,
) -> Result<(), AppError> {
    services::parts::delete(state.part_repo(), part_id).await?;
    Ok(())
}
