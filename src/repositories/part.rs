use std::future::Future;
use std::sync::Arc;

use crate::app::state::AppStateRef;
#[cfg(test)]
use crate::app::state::TestState;
use crate::models::part::{NewPart, Part, PartList, PartQuery};
use crate::{app::IsState, db::postgres::Db};
use anyhow::Result;
use mockall::automock;

pub struct PartRepositoryImpl {
    pool: Db,
}

impl PartRepositoryImpl {
    pub fn new(pool: Db) -> Self {
        Self { pool }
    }
}

pub(crate) trait HasPartRepo: IsState {
    type PartRepo: PartRepository + Send + Sync;
    fn part_repo(&self) -> Arc<Self::PartRepo>;
}

impl HasPartRepo for AppStateRef {
    type PartRepo = PartRepositoryImpl;
    fn part_repo(&self) -> Arc<Self::PartRepo> {
        self.part_repository.clone()
    }
}

#[cfg(test)]
impl<A: PartRepository + Send + Sync + 'static> HasPartRepo for TestState<A> {
    type PartRepo = A;
    fn part_repo(&self) -> Arc<Self::PartRepo> {
        self.0.clone()
    }
}

impl HasPartRepo for () {
    type PartRepo = MockPartRepository;
    fn part_repo(&self) -> Arc<Self::PartRepo> {
        Arc::new(MockPartRepository::new())
    }
}

#[automock]
pub trait PartRepository {
    fn find_all(&self, conditions: &PartQuery) -> impl Future<Output = Result<PartList>> + Send;
    fn create(&self, part_data: &NewPart) -> impl Future<Output = Result<Part>> + Send;
    fn update(&self, part_data: &Part) -> impl Future<Output = Result<Part>> + Send;
    fn delete(&self, part_id: i32) -> impl Future<Output = Result<u64>> + Send;
    fn find_by_id(&self, part_id: i32) -> impl Future<Output = Result<Part>> + Send;
}

impl PartRepository for PartRepositoryImpl {
    async fn find_all(&self, conditions: &PartQuery) -> Result<PartList> {
        let mut query = sqlx::query_as::<_, Part>("SELECT * FROM parts");
        if let Some(name) = &conditions.name {
            query = sqlx::query_as::<_, Part>("SELECT * FROM parts WHERE NAME LIKE $1")
                .bind(format!("%{}%", name))
        }
        let result = query.fetch_all(&*self.pool).await?;
        Ok(result)
    }

    async fn create(&self, part_data: &NewPart) -> Result<Part> {
        let created_part = sqlx::query_as::<_, Part>(
            r#"
            INSERT INTO parts (name, car_id)
            VALUES ($1, $2)
            RETURNING id, name, car_id
            "#,
        )
        .bind(&part_data.name)
        .bind(part_data.car_id)
        .fetch_one(&*self.pool)
        .await?;
        Ok(created_part)
    }

    async fn update(&self, part_data: &Part) -> Result<Part> {
        let updated_part = sqlx::query_as::<_, Part>(
            r#"
            UPDATE parts
            SET name = $2, car_id = $3
            WHERE id = $1
            RETURNING id, name, car_id
            "#,
        )
        .bind(part_data.id)
        .bind(&part_data.name)
        .bind(part_data.car_id)
        .fetch_one(&*self.pool)
        .await?;
        Ok(updated_part)
    }

    async fn delete(&self, part_id: i32) -> Result<u64> {
        let query = sqlx::query("DELETE FROM parts WHERE id = $1")
            .bind(part_id)
            .execute(&*self.pool)
            .await?;
        Ok(query.rows_affected())
    }

    async fn find_by_id(&self, part_id: i32) -> Result<Part> {
        let row = sqlx::query_as::<_, Part>("SELECT * FROM parts WHERE id = $1")
            .bind(part_id)
            .fetch_one(&*self.pool)
            .await?;
        Ok(row)
    }
}
