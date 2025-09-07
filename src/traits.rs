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
    
    async fn get_public_selfassessables(
        &self,
        pool: &MySqlPool,
        filter: SelfassessableFilter)
    -> Result<Vec<PublicSelfassessable>, sqlx::Error>;   
    
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
    async fn get_timetables(
        &self,
        pool: &MySqlPool,
        filter: TimetableFilter)
    -> Result<Vec<Timetable>, sqlx::Error>;
    async fn get_selfassessable_id(
        &self,
        pool: &MySqlPool,
        assessment_id: u64)
    -> Result<u64, sqlx::Error>;
    async fn get_assistance(
        &self,
        pool: &MySqlPool,
        filter: AssistanceFilter
    ) -> Result<Vec<Assistance>, sqlx::Error>;
    async fn get_disciplinary_sanction(
        &self,
        pool: &MySqlPool,
        filter: DisciplinarySanctionFilter
    ) -> Result<Vec<DisciplinarySanction>, sqlx::Error>;
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
    async fn get_is_selfassessable_answered(
        &self,
        pool: &MySqlPool,
        assessment_id: u64)
    -> Result<bool, sqlx::Error>;
    async fn get_is_homework_answered(
        &self,
        pool: &MySqlPool,
        homework_id: u64        
    ) -> Result<bool, sqlx::Error>;
    async fn post_assistance(
        &self,
        pool: &MySqlPool,
        assistance: NewAssistance
    ) -> HttpResponse;
    async fn post_disciplinary_sanction(
        &self,
        pool: &MySqlPool,
        disciplinary_sanction: NewDisciplinarySanction
    ) -> HttpResponse;
}


pub trait Delete {
    async fn delete_assessment(
        &self,
        pool: &MySqlPool,
        assessment_id: u64
    ) -> HttpResponse;
    async fn delete_grade(
        &self,
        pool: &MySqlPool,
        grade_id: u64
    ) -> HttpResponse;
    async fn delete_message(
        &self,
        pool: &MySqlPool,
        message_id: u64
    ) -> HttpResponse;
    async fn delete_personal_data(
        &self,
        pool: &MySqlPool,
        user_id: u64
    ) -> HttpResponse;
    async fn delete_profile_picture(
        &self,
        pool: &MySqlPool,
        user_id: u64)
    -> HttpResponse;
    async fn delete_subject_message(
        &self,
        pool: &MySqlPool,
        subject_message_id: u64
    ) -> HttpResponse;
    async fn delete_submission(
        &self,
        pool: &MySqlPool,
        submission_id: u64
    ) -> HttpResponse;
    async fn delete_assistance(
        &self,
        pool: &MySqlPool,
        assistance_id: u64,
        ) -> HttpResponse;
    async fn delete_disciplinary_sanction(
        &self,
        pool: &MySqlPool,
        disciplinary_sanction_id: u64
    ) -> HttpResponse;
}

pub trait Update {
    async fn update_assessment(
        &self,
        pool: &MySqlPool,
        assessment_id: u64,
        data: UpdateAssessment
    ) -> HttpResponse;
    async fn update_grade(
        &self,
        pool: &MySqlPool,
        grade_id: u64,
        data: UpdateGrade
    ) -> HttpResponse;
    async fn update_message(
        &self,
        pool: &MySqlPool,
        message_id: u64,
        data: UpdateMessage
    ) -> HttpResponse;
    async fn update_personal_data(
        &self,
        pool: &MySqlPool,
        user_id: u64,
        data: UpdatePersonalData
    ) -> HttpResponse;
    async fn update_profile_picture(
        &self,
        pool: &MySqlPool,
        user_id: u64,
        multipart: Multipart
    ) -> HttpResponse;
    async fn update_subject_message(
        &self,
        pool: &MySqlPool,
        subject_message_id: u64,
        data: UpdateSubjectMessage
    ) -> HttpResponse;
    async fn update_submission(
        &self,
        pool: &MySqlPool,
        submission_id: u64,
        data: UpdateSubmission
    ) -> HttpResponse;
    async fn update_assistance(
        &self,
        pool: &MySqlPool,
        assistance_id: u64,
        data: UpdateAssistance
    ) -> HttpResponse;
    async fn update_disciplinary_sanction(
        &self,
        pool: &MySqlPool,
        disciplinary_sanction_id: u64,
        data: UpdateDisciplinarySanction
    ) -> HttpResponse;
}

