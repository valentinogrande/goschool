use actix_web::web;

use crate::views::{
    // Assessments
    assessmets::{
        delete_assessment,
        get_assessments,
        post_assessment,
        update_assessment,
    },

    // Courses
    courses::get_courses,

    // Answer checks
    get_if_answered::{
        get_if_homework_answered,
        get_if_selfassessable_answered,
    },

    // Grades
    grades::{
        delete_grade,
        get_grades,
        post_grade,
        update_grade,
    },

    // Healthcheck
    health::health,

    // Auth
    login::login,
    logout::logout,
    register::{register, register_testing_users},
    verify_token::verify_token,

    // Roles
    role::get_role,
    roles::get_roles,

    // Selfassessables
    selfassessables::{
        get_selfassessables,
        get_selfassessables_responses,
        post_selfassessable_submission,
    },

    // Students
    students::get_students,

    // Subjects
    subjects::get_subjects,

    // Subject messages
    subject_messages::{
        delete_subject_message,
        get_subject_messages,
        post_subject_message,
        update_subject_message,
    },

    // Submissions
    submissions::post_homework_submission,

    // Timetables
    timetables::get_timetable,

    // Assistance
    assistance::{
        get_assisstance,
        post_assistance,
        update_assistance,
    },

    // Disciplinary sanctions
    disciplinary_sanctions::{
        get_disciplinary_sanction,
        post_disciplinary_sanction,
        update_disciplinary_sanction,
        delete_disciplinary_sanction,
    },

    // Personal data
    personal_data::{
        get_personal_data,
        get_public_personal_data,
    },

    // Profile pictures
    profile_pictures::{
        get_profile_picture,
        post_profile_picture,
    },

    // Messages
    messages::{
        delete_message,
        get_messages,
        post_message,
        update_message,
    },
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
        .service(health)
        .service(get_if_selfassessable_answered)
        .service(get_assisstance)
        .service(post_assistance)
        .service(update_assistance)
        .service(get_disciplinary_sanction)
        .service(post_disciplinary_sanction)
        .service(update_disciplinary_sanction)
        .service(delete_disciplinary_sanction);
    //.service(create_selfassessable_submission)
}
