use chrono::Utc;
use cron::Schedule;
use sqlx::MySqlPool;
use std::{str::FromStr, time::Duration};
use tokio::time::sleep;

async fn migrate_selfassessable_pending_grades(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    let insert_result = sqlx::query(
        r#"
        INSERT INTO grades (
            description,
            grade,
            student_id,
            subject_id,
            assessment_id,
            grade_type
        )
        SELECT
            'Autoevaluación cerrada',
            spg.grade,
            spg.student_id,
            s.id,
            a.id,
            'numerical'
        FROM selfassessable_pending_grades spg
        JOIN selfassessables sa ON sa.id = spg.selfassessable_id
        JOIN assessments a ON a.id = sa.assessment_id
        JOIN subjects s ON s.id = a.subject_id
        WHERE a.due_date < CURRENT_DATE();
        "#
    )
    .execute(pool)
    .await?;

    println!("Migrated {} pending selfassessables to grades", insert_result.rows_affected());

    sqlx::query(
        r#"
        DELETE spg FROM selfassessable_pending_grades spg
        JOIN selfassessables sa ON sa.id = spg.selfassessable_id
        JOIN assessments a ON a.id = sa.assessment_id
        WHERE a.due_date < CURRENT_DATE();
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn start_cron_task(pool: MySqlPool) {
    let schedule = Schedule::from_str("0 0/15 * * * * *").unwrap();
    let mut upcoming = schedule.upcoming(Utc);

    loop {
        if let Some(next) = upcoming.next() {
            let now = Utc::now();
            let wait = (next - now).to_std().unwrap_or(Duration::from_secs(0));
            sleep(wait).await;

            println!("Running migration task at {}", Utc::now());

            if let Err(e) = migrate_selfassessable_pending_grades(&pool).await {
                eprintln!("Error migrating pending grades: {}", e);
            }
        }
    }
}
