use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct UserFilter{
    pub course: Option<u64>,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GradeFilter{
    pub subject_id: Option<u64>,
    pub student_id: Option<u64>,
    pub description: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AssessmentFilter{
    pub subject_id: Option<u64>,
    pub task: Option<String>,
    pub due: Option<bool>,
}
