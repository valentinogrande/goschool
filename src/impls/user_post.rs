use actix_multipart::Multipart;
use actix_web::HttpResponse;
use futures::future::join_all;
use sqlx::MySqlPool;
use std::str;

use crate::filters::SelfassessableFilter;
use crate::functions::parse_multipart;
use crate::structs::{
    AssessmentType, MySelf, NewGrade, NewMessage, NewSubmissionSelfAssessable, Payload, Role,
};
use crate::traits::{Get, Post};

impl Post for MySelf {
    async fn post_assessment(&self, pool: &MySqlPool, payload: Payload) -> HttpResponse {
        match self.role {
            Role::teacher => {
                let teacher_subject: bool = match sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM subjects WHERE teacher_id = ? AND id = ?)",
                )
                .bind(self.id)
                .bind(payload.newtask.subject)
                .fetch_one(pool)
                .await
                {
                    Ok(s) => s,
                    Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
                };
                if !teacher_subject {
                    return HttpResponse::Unauthorized().finish();
                }
            }
            Role::admin => {}
            _ => return HttpResponse::Unauthorized().finish(),
        };

        if payload.newtask.type_ == AssessmentType::Selfassessable {
            let selfassessable = match &payload.newselfassessable {
                Some(a) => a,
                None => return HttpResponse::BadRequest().json("Missing selfassessable"),
            };

            if !(selfassessable.validate()) {
                return HttpResponse::BadRequest().json("Invalid selfassessable");
            }

            let insert_result = match sqlx::query(
                "INSERT INTO assessments (task, subject_id, type, due_date) VALUES (?, ?, ?, ?)",
            )
            .bind(&payload.newtask.task)
            .bind(payload.newtask.subject)
            .bind(&payload.newtask.type_)
            .bind(&payload.newtask.due_date)
            .execute(pool)
            .await
            {
                Ok(res) => res,
                Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
            };
            let assessment_id = insert_result.last_insert_id();

            let assessable =
                match sqlx::query("INSERT INTO selfassessables (assessment_id) VALUES (?)")
                    .bind(assessment_id)
                    .execute(pool)
                    .await
                {
                    Ok(r) => r,
                    Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
                };
            let assessable_id = assessable.last_insert_id();
            let mut queries = selfassessable.generate_query(assessable_id);

            let results = join_all(queries.iter_mut().map(|q| q.build().execute(pool))).await;
            for res in results {
                match res {
                    Ok(_) => {}
                    Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
                }
            }

            return HttpResponse::Created().finish();
        } else {
            let insert_result = sqlx::query(
                "INSERT INTO assessments (task, subject_id, type, due_date) VALUES (?, ?, ?, ?)",
            )
            .bind(&payload.newtask.task)
            .bind(payload.newtask.subject)
            .bind(&payload.newtask.type_)
            .bind(&payload.newtask.due_date)
            .execute(pool)
            .await;

            match insert_result {
                Ok(_) => HttpResponse::Created().finish(),
                Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
            }
        }
    }
    async fn post_grade(&self, pool: &MySqlPool, grade: NewGrade) -> HttpResponse {
        match self.role {
            Role::admin => {}
            Role::teacher => {
                let teacher_subject: bool = match sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM subjects WHERE teacher_id = ? AND id = ?)",
                )
                .bind(self.id)
                .bind(grade.subject)
                .fetch_one(pool)
                .await
                {
                    Ok(s) => s,
                    Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
                };
                if !teacher_subject {
                    return HttpResponse::Unauthorized().finish();
                }
            }
            _ => return HttpResponse::Unauthorized().finish(),
        };

        let course =
            match sqlx::query_scalar::<_, u64>("SELECT course_id FROM subjects WHERE id = ?")
                .bind(grade.subject)
                .fetch_one(pool)
                .await
            {
                Ok(c) => c,
                Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
            };

        let student_course: bool = match sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = ? AND course_id = ?)",
        )
        .bind(grade.student_id)
        .bind(course)
        .fetch_one(pool)
        .await
        {
            Ok(s) => s,
            Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
        };
        if !student_course {
            return HttpResponse::Unauthorized().finish();
        }

        if let Some(assessment_id) = grade.assessment_id {
            let assessment_verify: bool = match sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM assessments WHERE id = ? AND subject_id = ?)",
            )
            .bind(assessment_id)
            .bind(grade.subject)
            .fetch_one(pool)
            .await
            {
                Ok(s) => s,
                Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
            };
            if !assessment_verify {
                return HttpResponse::Unauthorized().finish();
            }
            let assessment_already_exixts: bool = match sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM grades WHERE assessment_id = ? AND student_id = ? )",
            )
            .bind(assessment_id)
            .bind(grade.student_id)
            .fetch_one(pool)
            .await
            {
                Ok(s) => s,
                Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
            };
            if assessment_already_exixts {
                return HttpResponse::Unauthorized().finish();
            }
            let result = sqlx::query("INSERT INTO grades (assessment_id, student_id, grade_type, description, grade, subject_id) VALUES (?, ?, ?, ?, ?, ?)")
                .bind(assessment_id)
                .bind(grade.student_id)
                .bind(&grade.grade_type)
                .bind(&grade.description)
                .bind(grade.grade)
                .bind(grade.subject)
                .execute(pool)
                .await;
            if result.is_err() {
                return HttpResponse::InternalServerError().finish();
            } else {
                return HttpResponse::Created().finish();
            }
        }
        let result = sqlx::query("INSERT INTO grades (student_id, grade_type, description, grade, subject_id) VALUES (?, ?, ?, ?, ?)")
            .bind(grade.student_id)
            .bind(&grade.grade_type)
            .bind(&grade.description)
            .bind(grade.grade)
            .bind(grade.subject)
            .execute(pool)
            .await;
        if result.is_err() {
            return HttpResponse::InternalServerError().finish();
        } else {
            return HttpResponse::Created().finish();
        }
    }
    async fn post_message(&self, pool: &MySqlPool, message: NewMessage) -> HttpResponse {
        // cheking if courses are valid
        let courses: Vec<u64> = message
            .courses
            .split(',')
            .filter_map(|s| s.trim().parse::<u64>().ok())
            .collect();
        for course in courses.iter() {
            if *course <= 0 || *course > 36 {
                return HttpResponse::BadRequest().json("Invalid courses");
            }
        }

        match self.role {
            Role::admin => {}
            Role::preceptor => {
                let preceptor_courses: Vec<u64> = match self.get_courses(pool).await {
                    Ok(c) => c.iter().map(|c| c.id).collect(),
                    Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
                };
                if !preceptor_courses
                    .iter()
                    .all(|&course| courses.contains(&course))
                {
                    return HttpResponse::Unauthorized().finish();
                }
            }
            _ => {}
        };

        let message_id =
            match sqlx::query("INSERT INTO messages (message, sender_id, title) VALUES (?, ?, ?)")
                .bind(&message.message)
                .bind(self.id)
                .bind(&message.title)
                .execute(pool)
                .await
            {
                Ok(ref result) => result.last_insert_id(),
                Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
            };

        for course in courses.iter() {
            let _insert_result = match sqlx::query(
                "INSERT INTO message_courses (course_id, message_id) VALUES (?,?)",
            )
            .bind(course)
            .bind(message_id)
            .execute(pool)
            .await
            {
                Ok(r) => r,
                Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
            };
        }
        HttpResponse::Created().finish()
    }

    async fn post_profile_picture(
        &self,
        pool: &MySqlPool,
        multipart: actix_multipart::Multipart,
    ) -> HttpResponse {
        let hashmap = match parse_multipart(
            multipart,
            Some(&["jpg", "jpeg", "png"]),
            Some(&["image/jpeg", "image/png"]),
            "uploads/profile_pictures",
        )
        .await
        {
            Ok(h) => h,
            Err(e) => return HttpResponse::BadRequest().json(format!("Invalid upload: {}", e)),
        };

        let file_name = hashmap
            .get("file")
            .and_then(|bytes| str::from_utf8(bytes).ok());

        dbg!(&file_name);

        let result = sqlx::query("UPDATE users SET photo = ? WHERE id = ?")
            .bind(file_name)
            .bind(self.id)
            .execute(pool)
            .await;

        if result.is_err() {
            return HttpResponse::InternalServerError().finish();
        }

        HttpResponse::Created().finish()
    }
    async fn post_submission(&self, pool: &MySqlPool, multipart: Multipart) -> HttpResponse {
        match self.role {
            Role::student => {}
            _ => return HttpResponse::Unauthorized().finish(),
        };

        let user_course = match self.get_courses(pool).await {
            Ok(c) if !c.is_empty() => c[0].id,
            Ok(_) => return HttpResponse::NotFound().json("No course found"),
            Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
        };

        let hashmap = match parse_multipart(
            multipart,
            Some(&["pdf", "docx"]),
            Some(&[
                "application/pdf",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ]),
            "uploads/submissions",
        )
        .await
        {
            Ok(h) => h,
            Err(e) => return HttpResponse::BadRequest().json(format!("Invalid upload: {}", e)),
        };

        let homework_id = match hashmap
            .get("homework_id")
            .and_then(|bytes| str::from_utf8(bytes).ok())
            .and_then(|s| s.parse::<u64>().ok())
        {
            Some(id) => id,
            None => return HttpResponse::BadRequest().json("Missing or invalid task_id"),
        };

        let file_name = match hashmap.get("file") {
            Some(f) => f,
            None => return HttpResponse::BadRequest().json("Missing file"),
        };

        let res: (String, u64) =
            match sqlx::query_as("SELECT type, subject_id FROM assessments WHERE id = ?")
                .bind(homework_id)
                .fetch_one(pool)
                .await
            {
                Ok(g) => g,
                Err(_) => {
                    return HttpResponse::InternalServerError().finish();
                }
            };

        if res.0 != "homework" {
            return HttpResponse::BadRequest().body("submission are only valid for homeworks");
        }
        let task_course =
            match sqlx::query_scalar::<_, u64>("SELECT course_id FROM subjects WHERE id = ?")
                .bind(res.1)
                .fetch_one(pool)
                .await
            {
                Ok(g) => g,
                Err(_) => {
                    return HttpResponse::InternalServerError().finish();
                }
            };

        if task_course != user_course {
            return HttpResponse::Unauthorized().finish();
        }

        let already_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM homework_submissions WHERE student_id = ? AND task_id = ?)"
        )
        .bind(self.id)
        .bind(homework_id)
        .fetch_one(pool)
        .await;

        match already_exists {
            Ok(true) => {
                return HttpResponse::BadRequest().body("You already submitted this homework");
            }
            Ok(false) => {}
            Err(_) => {
                return HttpResponse::InternalServerError().finish();
            }
        }

        let result = sqlx::query(
            "INSERT INTO homework_submissions (path, student_id, task_id) VALUES (?, ?, ?)",
        )
        .bind(file_name)
        .bind(self.id)
        .bind(homework_id)
        .execute(pool)
        .await;

        if result.is_err() {
            return HttpResponse::InternalServerError().finish();
        }

        return HttpResponse::Created().body("Submission created");
    }

    async fn post_submission_selfassessable(
        &self,
        pool: &MySqlPool,
        task_submission: NewSubmissionSelfAssessable,
    ) -> HttpResponse {
        if self.role != Role::student {
            return HttpResponse::Unauthorized().finish();
        }

        let user_course =
            match sqlx::query_scalar::<_, u64>("SELECT course_id FROM users WHERE id = ?")
                .bind(self.id)
                .fetch_one(pool)
                .await
            {
                Ok(course_id) => course_id,
                Err(_) => return HttpResponse::InternalServerError().finish(),
            };

        let (assessment_type, subject_id): (String, u64) =
            match sqlx::query_as("SELECT type, subject_id FROM assessments WHERE id = ?")
                .bind(task_submission.assessment_id)
                .fetch_one(pool)
                .await
            {
                Ok(res) => res,
                Err(_) => return HttpResponse::InternalServerError().finish(),
            };

        if assessment_type != "selfassessable" {
            return HttpResponse::BadRequest().body("submission are only valid for selfassables");
        }

        let assessable_course =
            match sqlx::query_scalar::<_, u64>("SELECT course_id FROM subjects WHERE id = ?")
                .bind(subject_id)
                .fetch_one(pool)
                .await
            {
                Ok(course_id) => course_id,
                Err(_) => return HttpResponse::InternalServerError().finish(),
            };

        if assessable_course != user_course {
            return HttpResponse::Unauthorized().finish();
        }

        let selfassessable_id = match sqlx::query_scalar::<_, u64>(
            "SELECT id FROM selfassessables WHERE assessment_id = ?",
        )
        .bind(task_submission.assessment_id)
        .fetch_one(pool)
        .await
        {
            Ok(id) => id,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };

        let already_exists = match sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM selfassessable_submissions WHERE student_id = ? AND selfassessable_id = ?)"
        )
        .bind(self.id)
        .bind(selfassessable_id)
        .fetch_one(pool)
        .await {
            Ok(exists) => exists,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };

        if already_exists {
            return HttpResponse::BadRequest().body("You already submitted this homework");
        }

        let answers = task_submission.answers;

        let filter = SelfassessableFilter {
            selfassessable_id: Some(selfassessable_id),
        };

        let assessable_task = match self.get_selfassessables(&pool, filter).await {
            Ok(u) => u[0].clone(),
            Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
        };

        let corrects = assessable_task.correct.split(',').collect::<Vec<_>>();

        let mut grade = 0;

        dbg!(&answers);
        dbg!(&corrects);

        for (answer, correct) in answers.iter().zip(corrects.iter()) {
            if answer == correct {
                grade += 1;
            }
        }
        let percentage = grade as f64 / answers.len() as f64;
        dbg!(&percentage);

        let result = sqlx::query("INSERT INTO selfassessable_pending_grades (selfassessable_id, student_id, grade) VALUES (?, ?, ?)")
            .bind(assessable_task.id)
            .bind(self.id)
            .bind(percentage * 10.0)
            .execute(pool)
            .await;
        if result.is_err() {
            return HttpResponse::InternalServerError().finish();
        }

        match sqlx::query(
            "INSERT INTO selfassessable_submissions (selfassessable_id, student_id, answers) VALUES (?, ?, ?)"
        )
        .bind(selfassessable_id)
        .bind(self.id)
        .bind(answers.join(","))
        .execute(pool)
        .await {
            Ok(_) => HttpResponse::Created().finish(),
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
    }
    async fn post_subject_messages(&self, pool: &MySqlPool, multipart: Multipart) -> HttpResponse {
        // Parse multipart with allowed extensions and MIME types
        let hashmap = match parse_multipart(
            multipart,
            Some(&["pdf", "docx"]),
            Some(&[
                "application/pdf",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ]),
            "uploads/files",
        )
        .await
        {
            Ok(h) => h,
            Err(e) => return HttpResponse::BadRequest().json(format!("Invalid upload: {}", e)),
        };

        // Extract and parse subject_id
        let subject_id = match hashmap
            .get("subject_id")
            .and_then(|bytes| str::from_utf8(bytes).ok())
            .and_then(|s| s.parse::<u64>().ok())
        {
            Some(id) => id,
            None => return HttpResponse::BadRequest().json("Missing or invalid subject_id"),
        };

        // Extract and parse type
        let type_ = match hashmap
            .get("type")
            .and_then(|bytes| str::from_utf8(bytes).ok())
        {
            Some(t) => t,
            None => "message",
        };

        let title = match hashmap
            .get("title")
            .and_then(|bytes| str::from_utf8(bytes).ok())
        {
            Some(t) => t,
            None => "",
        };

        // Extract content or file name
        let (content_or_file, is_text) = match type_ {
            "file" => match hashmap.get("file") {
                Some(f) => (f, false),
                None => return HttpResponse::BadRequest().json("Missing file"),
            },
            "message" => match hashmap.get("content") {
                Some(c) => match str::from_utf8(c) {
                    Ok(s) => (&s.as_bytes().to_vec(), true),
                    Err(_) => return HttpResponse::BadRequest().json("Invalid content encoding"),
                },
                None => return HttpResponse::BadRequest().json("Missing content"),
            },
            _ => return HttpResponse::BadRequest().json("Invalid type"),
        };

        // Authorization
        match self.role {
            Role::student | Role::father | Role::preceptor => {
                return HttpResponse::Unauthorized().finish();
            }
            Role::teacher => {
                let authorized: bool = match sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM subjects WHERE teacher_id = ? AND id = ?)",
                )
                .bind(self.id)
                .bind(subject_id)
                .fetch_one(pool)
                .await
                {
                    Ok(result) => result,
                    Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
                };

                if !authorized {
                    return HttpResponse::Unauthorized().finish();
                }
            }
            Role::admin => {}
        }

        // Insert message into database
        let insert_result = if is_text {
            // For text type, convert bytes back to string for database
            let content_str = str::from_utf8(content_or_file).unwrap_or("");
            sqlx::query(
                "INSERT INTO subject_messages (subject_id, sender_id, type, content, title) VALUES (?, ?, ?, ?, ?)"
            )
                .bind(subject_id)
                .bind(self.id)
                .bind(type_)
                .bind(content_str)
                .bind(title)
                .execute(pool)
                .await
        } else {
            // For file type, use the bytes directly (it's a file path)
            let file_path = str::from_utf8(content_or_file).unwrap_or("");
            sqlx::query(
            "INSERT INTO subject_messages (subject_id, sender_id, type, content, title) VALUES (?, ?, ?, ?, ?)"
        )
            .bind(subject_id)
            .bind(self.id)
            .bind(type_)
                .bind(file_path)
            .bind(title)
            .execute(pool)
                .await
        };

        match insert_result {
            Ok(_) => HttpResponse::Created().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    }
    async fn get_is_selfassessable_answered(
        &self,
        pool: &MySqlPool,
        selfassessable_id: u64,
    ) -> anyhow::Result<bool, sqlx::Error> {
        let res = sqlx::query_scalar("SELECT EXISTS (SELECT * FROM selfassessable_submissions WHERE student_id = ? AND selfassessable_id = ?)")
            .bind(self.id)
            .bind(selfassessable_id)
            .fetch_one(pool)
            .await;
        res
    }
    async fn get_is_homework_answered(
        &self,
        pool: &MySqlPool,
        homework_id: u64,
    ) -> anyhow::Result<bool, sqlx::Error> {
        let res = sqlx::query_scalar("SELECT EXISTS (SELECT * FROM homework_submissions WHERE student_id = ? AND task_id = ?)")
            .bind(self.id)
            .bind(homework_id)
            .fetch_one(pool)
            .await;
        res
    }
}
