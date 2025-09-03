use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AssessmentFilter {
    pub subject_id: Option<u64>,
    pub task: Option<String>,
    pub due: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct GradeFilter {
    pub subject_id: Option<u64>,
    pub student_id: Option<u64>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct MessageFilter {
    pub sender_id: Option<u64>,
    pub title: Option<String>,
    pub course_id: Option<u64>,
}


#[derive(Serialize, Deserialize)]
pub struct TimetableFilter {
    pub teacher_id: Option<u64>,
    pub course_id: Option<u64>,
    pub subject_id: Option<u64>,
    pub day: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SubjectFilter {
    pub teacher_id: Option<u64>,
    pub course_id: Option<u64>,
    pub name: Option<String>,
    pub subject_id: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct SubjectMessageFilter {
    pub subject_message_id: Option<u64>,
    pub sender_id: Option<u64>,
    pub subject_id: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct UserFilter {
    pub course: Option<u64>,
    pub name: Option<String>,
    pub user_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct SelfassessableFilter {
    pub assessment_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AssistanceFilter {
    pub assistance_id: Option<u64>,
    pub student_id: Option<u64>,
    pub presence: Option<String>,
    pub date: Option<NaiveDate>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DisciplinarySanctionFilter {
    pub disciplinary_sanction_id: Option<u64>,
    pub student_id: Option<u64>,
    pub sanction_type: Option<String>,
}
