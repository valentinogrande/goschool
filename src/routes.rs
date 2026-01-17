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
    submissions::{post_homework_submission, update_submission, delete_submission},

    // Timetables
    timetables::{
        get_timetable,
        post_timetable,
        update_timetable,
        delete_timetable,
    },

    // Assistance
    assistance::{
        get_assisstance,
        post_assistance,
        update_assistance,
        delete_assistance,
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
        update_personal_data,
        delete_personal_data,
    },

    // Profile pictures
    profile_pictures::{
        get_profile_picture,
        post_profile_picture,
        update_profile_picture,
        delete_profile_picture,
        update_self_profile_picture,
    },

    // Messages
    messages::{
        delete_message,
        get_messages,
        post_message,
        update_message,
    },

    // Chat API
    chat_api::{
        get_user_chats,
        create_chat,
        get_chat_messages,
        send_message,
        add_participants,
        remove_participant,
        get_available_users,
        upload_chat_file,
        mark_chat_as_read,
    },
};

// WebSocket handler
use crate::websocket::chat_websocket;


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
        .service(update_personal_data)
        .service(delete_personal_data)
        .service(get_profile_picture)
        .service(update_profile_picture)
        .service(delete_profile_picture)
        .service(update_self_profile_picture)
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
        .service(update_submission)
        .service(delete_submission)
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
        .service(post_timetable)
        .service(update_timetable)
        .service(delete_timetable)
        .service(verify_token)
        .service(get_if_homework_answered)
        .service(health)
        .service(get_if_selfassessable_answered)
        .service(get_assisstance)
        .service(post_assistance)
        .service(update_assistance)
        .service(delete_assistance)
        .service(get_disciplinary_sanction)
        .service(post_disciplinary_sanction)
        .service(update_disciplinary_sanction)
        .service(delete_disciplinary_sanction)
        // Chat WebSocket
        .service(chat_websocket)
        // Chat REST API
        .service(get_user_chats)
        .service(create_chat)
        .service(get_chat_messages)
        .service(send_message)
        .service(add_participants)
        .service(remove_participant)
        .service(get_available_users)
        .service(upload_chat_file)
        .service(mark_chat_as_read);
}
