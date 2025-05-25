use actix_web::web;

use crate::views::{
    get_courses::get_courses,
    get_subjects::get_subjects,
    assign_grade::assign_grade,
    create_assessment::create_assessment,
    create_homework_submission::create_submission,
    get_assessmets::get_assessments,
    get_grades::get_grades,
    get_personal_data::get_personal_data,
    get_profile_picture::get_profile_picture,
    get_role::get_role,
    get_roles::get_roles,
    get_messages::get_messages,
    login::login,
    logout::logout,
    post_message::post_message,
    register::register,
    register_testing_users::register_users,
    upload_profile_picture::upload_profile_picture,
    verify_token::verify_token,
    create_submission_selfassable::create_selfassessable_submission,
};


pub fn register_services(cfg: &mut web::ServiceConfig) {
    cfg.service(register)
        .service(verify_token)
        .service(login)
        .service(logout)
        .service(create_submission)
        .service(get_assessments)
        .service(get_grades)
        .service(get_personal_data)
        .service(create_assessment)
        .service(upload_profile_picture)
        .service(get_profile_picture)
        .service(get_role)
        .service(get_roles)
        .service(post_message)
        .service(get_messages)
        .service(assign_grade)
        .service(create_selfassessable_submission)
        .service(get_subjects)
        .service(get_courses)
        .service(register_users);
}
