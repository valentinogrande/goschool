use sqlx::MySqlPool;
use actix_web::{web, HttpResponse};
use anyhow::Result;
use actix_multipart::Multipart;

use crate::filters::{GradeFilter, UserFilter, SubjectFilter, AssessmentFilter, MessageFilter};
use crate::structs::{Assessment, Course, Grade, Message, NewGrade, NewMessage, Payload, PersonalData, Role, Subject}; 

pub trait New {
    fn new(id: u64, role: Role) -> Self;
}
pub trait Get {
    async fn get_students(
        &self,
        pool: web::Data<MySqlPool>,
        filter: Option<UserFilter>)
    -> Result<Vec<u64>, sqlx::Error>;

    async fn get_courses(
        &self,
        pool: &MySqlPool)
    -> Result<Vec<Course>, sqlx::Error>;
    
    async fn get_grades(
        &self,
        pool: &MySqlPool,
        filter: Option<GradeFilter>)
    -> Result<Vec<Grade>, sqlx::Error>;
    
    async fn get_subjects(
        &self,
        pool: &MySqlPool,
        filter: Option<SubjectFilter>
    ) -> Result<Vec<Subject>, sqlx::Error>;

    async fn get_assessments(
        &self,
        pool: &MySqlPool,
        filter: Option<AssessmentFilter>,
        subject_filter: Option<SubjectFilter>,
        person_filter: Option<UserFilter>)
    -> Result<Vec<Assessment>, sqlx::Error>;
    
    async fn get_messages(
        &self,
        pool: &MySqlPool,
        filter: Option<MessageFilter>)
    -> Result<Vec<Message>, sqlx::Error>;
    
    async fn get_personal_data(
        &self,
        pool: &MySqlPool)
    -> Result<PersonalData, sqlx::Error>;
    
    async fn get_profile_picture(
        &self,
        pool: &MySqlPool)
    -> Result<String, sqlx::Error>;
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
}
