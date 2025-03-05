use std::future::Future;
use std::sync::Arc;

use crate::app::state::AppStateRef;
#[cfg(test)]
use crate::app::state::TestState;
use crate::app::IsState;
use crate::db::postgres::Db;
use crate::models::car::{Car, CarList, CarQuery, NewCar};
use anyhow::Result;
use mockall::automock;

#[derive(Clone)]
pub struct CarRepositoryImpl {
    pool: Db,
}
impl CarRepositoryImpl {
    pub fn new(pool: Db) -> Self {
        Self { pool }
    }
}

pub(crate) trait HasCarRepo: IsState {
    type CarRepo: CarRepository + Send + Sync;
    fn car_repo(&self) -> Arc<Self::CarRepo>;
}

impl HasCarRepo for AppStateRef {
    type CarRepo = CarRepositoryImpl;
    fn car_repo(&self) -> Arc<Self::CarRepo> {
        self.car_repository.clone()
    }
}

#[cfg(test)]
impl<A: CarRepository + Send + Sync + 'static> HasCarRepo for TestState<A> {
    type CarRepo = A;
    fn car_repo(&self) -> Arc<Self::CarRepo> {
        self.0.clone()
    }
}

impl HasCarRepo for () {
    type CarRepo = MockCarRepository;
    fn car_repo(&self) -> Arc<Self::CarRepo> {
        Arc::new(MockCarRepository::new())
    }
}

#[automock]
pub trait CarRepository: Send + Sync + 'static {
    fn find_all(&self, conditions: &CarQuery) -> impl Future<Output = Result<CarList>> + Send;
    fn create(&self, car_data: &NewCar) -> impl Future<Output = Result<Car>> + Send;
    fn update(&self, car_data: &Car) -> impl Future<Output = Result<Car>> + Send;
    fn delete(&self, car_id: i32) -> impl Future<Output = Result<u64>> + Send;
    fn find_by_id(&self, car_id: i32) -> impl Future<Output = Result<Car>> + Send;
}

impl CarRepository for CarRepositoryImpl {
    async fn find_all(&self, conditions: &CarQuery) -> Result<CarList> {
        let mut query = sqlx::query_as::<_, Car>("SELECT * FROM cars");
        if let Some(name) = &conditions.name {
            query = sqlx::query_as::<_, Car>("SELECT * FROM cars WHERE NAME LIKE $1")
                .bind(format!("%{}%", name))
        }
        let result = query.fetch_all(&*self.pool).await?;
        Ok(result)
    }

    async fn create(&self, car_data: &NewCar) -> Result<Car> {
        let created_car = sqlx::query_as::<_, Car>(
            r#"
            INSERT INTO cars (name, color, year)
            VALUES ($1, $2, $3)
            RETURNING id, name, color, year
            "#,
        )
        .bind(&car_data.name)
        .bind(&car_data.color)
        .bind(car_data.year)
        .fetch_one(&*self.pool)
        .await?;
        Ok(created_car)
    }

    async fn update(&self, car_data: &Car) -> Result<Car> {
        let updated_car = sqlx::query_as::<_, Car>(
            r#"
            UPDATE cars
            SET name = $2, color = $3, year = $4
            WHERE id = $1
            RETURNING id, name, color, year
            "#,
        )
        .bind(car_data.id)
        .bind(&car_data.name)
        .bind(&car_data.color)
        .bind(car_data.year)
        .fetch_one(&*self.pool)
        .await?;
        Ok(updated_car)
    }

    async fn delete(&self, car_id: i32) -> Result<u64> {
        let query = sqlx::query("DELETE FROM cars WHERE id = $1")
            .bind(car_id)
            .execute(&*self.pool)
            .await?;
        Ok(query.rows_affected())
    }

    async fn find_by_id(&self, car_id: i32) -> Result<Car> {
        let row = sqlx::query_as::<_, Car>("SELECT * FROM cars WHERE id = $1")
            .bind(car_id)
            .fetch_one(&*self.pool)
            .await?;
        Ok(row)
    }
}

#[cfg(test)]
mod tests {

    use crate::services::mock_result;

    use super::*;
    use mockall::predicate;
    #[tokio::test]
    async fn test_find_all_cars() {
        let mut mock_repo = MockCarRepository::new();
        let conditions = CarQuery {
            name: Some("Tesla".to_string()),
        };
        let expected_cars = vec![
            Car {
                id: 1,
                name: "Tesla Model S".to_string(),
                color: None,
                year: None,
            },
            Car {
                id: 2,
                name: "Tesla Model 3".to_string(),
                color: None,
                year: None,
            },
        ];

        mock_repo
            .expect_find_all()
            .with(predicate::eq(conditions.clone()))
            .times(1)
            .returning(move |_| mock_result(expected_cars.clone()));

        let result = mock_repo.find_all(&conditions).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
}
