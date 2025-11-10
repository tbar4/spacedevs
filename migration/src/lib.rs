pub use sea_orm_migration::prelude::*;

mod m20251110_032202_create_space_devs_base;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20251110_032202_create_space_devs_base::Migration)]
    }
}
