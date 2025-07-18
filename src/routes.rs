use actix_web::web;

use crate::views::{
    assessmets::{get_assessments, post_assessment, update_assessment, delete_assessment},
    courses::get_courses,
    get_if_answered::{get_if_homework_answered, get_if_selfassessable_answered},
    grades::{get_grades, post_grade, update_grade, delete_grade},
    login::login,
    logout::logout,
    messages::{get_messages, post_message, update_message, delete_message},
    personal_data::{get_personal_data, get_public_personal_data},
    profile_pictures::{get_profile_picture, post_profile_picture},
    register::{register, register_testing_users},
    role::get_role,
    roles::get_roles,
    selfassessables::{
        get_selfassessables, get_selfassessables_responses, post_selfassessable_submission,
    },
    students::get_students,
    subject_messages::{get_subject_messages, post_subject_message, update_subject_message, delete_subject_message},
    subjects::get_subjects,
    submissions::post_homework_submission,
    verify_token::verify_token,
    timetables::get_timetable,
};

pub fn register_services(cfg: &mut web::ServiceConfig) {
    cfg.service(get_assessments)
        .service(get_courses)
        .service(get_grades)
        .service(post_grade)
        .service(update_grade)
        .service(delete_grade)
        .service(get_messages)
        .service(post_message)
        .service(update_message)
        .service(delete_message)
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
        .service(update_assessment)
        .service(delete_assessment)
        .service(post_homework_submission)
        .service(post_selfassessable_submission)
        .service(get_selfassessables)
        .service(get_selfassessables_responses)
        .service(post_message)
        .service(post_subject_message)
        .service(get_subject_messages)
        .service(update_subject_message)
        .service(delete_subject_message)
        .service(post_profile_picture)
        .service(register)
        .service(register_testing_users)
        .service(get_timetable)
        .service(verify_token)
        .service(get_if_homework_answered)
        .service(get_if_selfassessable_answered);

    //.service(create_selfassessable_submission)
}
