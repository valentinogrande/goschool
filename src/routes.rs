use actix_web::web;

use crate::views::{
    courses::get_courses,
    subjects::get_subjects,
    submissions::post_homework_submission,
    assessmets::{get_assessments, post_assessment},
    grades::{get_grades, post_grade},
    personal_data::get_personal_data,
    profile_pictures::{get_profile_picture, post_profile_picture},
    role::get_role,
    roles::get_roles,
    messages::{get_messages, post_message},
    login::login,
    logout::logout,
    register::{register, register_testing_users},
    verify_token::verify_token,
    //create_submission_selfassable::create_selfassessable_submission,
};


pub fn register_services(cfg: &mut web::ServiceConfig) {
    cfg.service(register)
        .service(verify_token)
        .service(login)
        .service(logout)
        .service(post_homework_submission)
        .service(get_assessments)
        .service(get_grades)
        .service(get_personal_data)
        .service(post_assessment)
        .service(post_profile_picture)
        .service(get_profile_picture)
        .service(get_role)
        .service(get_roles)
        .service(post_message)
        .service(get_messages)
        .service(post_grade)
        //.service(create_selfassessable_submission)
        .service(get_subjects)
        .service(get_courses)
        .service(register_testing_users);
}
