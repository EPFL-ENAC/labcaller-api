pub use sea_orm_migration::prelude::*;

mod m20240926_143036_create_submission_table;
mod m20241009_142236_create_system_status_table;
mod m20241010_073350_create_input_objects;
mod m20241029_154332_create_runstatus_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240926_143036_create_submission_table::Migration),
            Box::new(m20241009_142236_create_system_status_table::Migration),
            Box::new(m20241010_073350_create_input_objects::Migration),
            Box::new(m20241029_154332_create_runstatus_table::Migration),
        ]
    }
}
