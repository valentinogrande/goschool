use actix_web::web;

use crate::views::{
    assessmets::{get_assessments, post_assessment},
    courses::get_courses,
    get_if_answered::{get_if_homework_answered, get_if_selfassessable_answered},
    grades::{get_grades, post_grade},
    login::login,
    logout::logout,
    messages::{get_messages, post_message},
    personal_data::{get_personal_data, get_public_personal_data},
    profile_pictures::{get_profile_picture, post_profile_picture},
    register::{register, register_testing_users},
    role::get_role,
    roles::get_roles,
    selfassessables::{
        get_selfassessables, get_selfassessables_responses, post_selfassessable_submission,
    },
    students::get_students,
    subject_messages::{get_subject_messages, post_subject_message},
    subjects::get_subjects,
    submissions::post_homework_submission,
    verify_token::verify_token,
};

pub fn register_services(cfg: &mut web::ServiceConfig) {
    cfg.service(get_assessments)
        .service(get_courses)
        .service(get_grades)
        .service(get_messages)
        .service(get_personal_data)
        .service(get_public_personal_data)
        .service(get_profile_picture)
        .service(get_role)
        .service(get_roles)
        .service(get_students)
        .service(get_subjects)
        .service(login)
        .service(logout)
        .service(post_assessment)
        .service(post_grade)
        .service(post_homework_submission)
        .service(post_selfassessable_submission)
        .service(get_selfassessables)
        .service(get_selfassessables_responses)
        .service(post_message)
        .service(post_subject_message)
        .service(get_subject_messages)
        .service(post_profile_picture)
        .service(register)
        .service(register_testing_users)
        .service(verify_token)
        .service(get_if_homework_answered)
        .service(get_if_selfassessable_answered);

    //.service(create_selfassessable_submission)
}
