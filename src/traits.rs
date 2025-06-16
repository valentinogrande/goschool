use sqlx::MySqlPool;
use actix_web::{web, HttpResponse};
use anyhow::Result;
use actix_multipart::Multipart;

use crate::filters::*;
use crate::structs:: *;

pub trait New {
    fn new(id: u64, role: Role) -> Self;
}
pub trait Get {
    async fn get_students(
        &self,
        pool: web::Data<MySqlPool>,
        filter: UserFilter)
    -> Result<Vec<u64>, sqlx::Error>;

    async fn get_courses(
        &self,
        pool: &MySqlPool)
    -> Result<Vec<Course>, sqlx::Error>;
    
    async fn get_grades(
        &self,
        pool: &MySqlPool,
        filter: GradeFilter)
    -> Result<Vec<Grade>, sqlx::Error>;
    
    async fn get_subjects(
        &self,
        pool: &MySqlPool,
        filter: SubjectFilter
    ) -> Result<Vec<Subject>, sqlx::Error>;

    async fn get_assessments(
        &self,
        pool: &MySqlPool,
        filter: AssessmentFilter,
        subject_filter: SubjectFilter,
        person_filter: UserFilter)
    -> Result<Vec<Assessment>, sqlx::Error>;
    
    async fn get_messages(
        &self,
        pool: &MySqlPool,
        filter: MessageFilter)
    -> Result<Vec<Message>, sqlx::Error>;
    
    async fn get_personal_data(
        &self,
        pool: &MySqlPool)
    -> Result<PersonalData, sqlx::Error>;
    
    async fn get_profile_picture(
        &self,
        pool: &MySqlPool)
    -> Result<String, sqlx::Error>;
    async fn get_selfassessables(
        &self,
        pool: &MySqlPool,
        filter: SelfassessableFilter)
    -> Result<Vec<Selfassessable>, sqlx::Error>;   
    async fn get_selfassessables_responses(
        &self,
        pool: &MySqlPool,
        filter: SelfassessableFilter)
     -> Result<Vec<SelfassessableResponse>, sqlx::Error>;
    async fn get_public_personal_data(
        &self,
        pool: &MySqlPool,
        filter: UserFilter)
    -> Result<Vec<PublicPersonalData>, sqlx::Error>;
    async fn get_pending_selfassessables_grades(
        &self,
        pool: &MySqlPool,
        filter: SelfassessableFilter)
    -> Result<Vec<PendingSelfassessableGrade>, sqlx::Error>;
    async fn get_subject_messages(
        &self,
        pool: &MySqlPool,
        filter: SubjectMessageFilter)
    -> Result<Vec<SubjectMessage>, sqlx::Error>;
}

pub trait Post  {
    async fn post_assessment(
        &self,
        pool: &MySqlPool,
        payload: Payload,
) -> HttpResponse;

    async fn post_grade(
        &self,
        pool: &MySqlPool,
        grade: NewGrade,
    ) -> HttpResponse;
    async fn post_message(
        &self,
        pool: &MySqlPool,
        message: NewMessage,
    ) -> HttpResponse;
    async fn post_profile_picture(
        &self,
        pool: &MySqlPool,
        task_submission: Multipart
    ) -> HttpResponse;
    async fn post_submission(
        &self,
        pool: &MySqlPool,
        multipart: Multipart
    ) -> HttpResponse;
    async fn post_submission_selfassessable(
        &self,
        pool: &MySqlPool,
        task_submission: NewSubmissionSelfAssessable,
    ) -> HttpResponse;
    async fn post_subject_messages(
        &self,
        pool: &MySqlPool,
        multipart: Multipart
    ) -> HttpResponse;
}
