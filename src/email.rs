use std::env;
use lettre::message::{header::ContentType, Mailbox, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{SmtpTransport, Transport};
use std::fs::read_to_string;
use tera::{Context, Tera};


pub async fn send_grade_email(
    reply_to: Vec<String>,
    subject: &str,
    sender_name: &str,
    student_name: &str,
    grade: &str,
) {

    let from_str = env::var("EMAIL_FROM").expect("EMAIL_FROM must be set");
    let from: Mailbox = from_str.parse().expect("Invalid EMAIL_FROM format");
    let base_dir = env::var("BASE_PATH").expect("BASE_PATH must be set");

    let template_path = format!("{}/src/email_templates/grade_submitted.html", base_dir);
    let template_str = match read_to_string(&template_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error cargando template: {}", e);
            return;
        }
    };

    let mut tera = Tera::default();
    tera.add_raw_template("grade_submitted", &template_str).expect("Template inválido");

    let mut context = Context::new();
    context.insert("sender_name", sender_name);
    context.insert("student_name", student_name);
    context.insert("subject", subject);
    context.insert("grade", grade);

    let body = match tera.render("grade_submitted", &context) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error renderizando template: {}", e);
            return;
        }
    };

    let email_subject = "Grade Submitted";

    let credentials = Credentials::new(
        env::var("EMAIL_USERNAME").expect("EMAIL_USERNAME must be set"),
        env::var("EMAIL_PASSWORD").expect("EMAIL_PASSWORD must be set"),
    );
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(credentials)
        .build();

    let header = ContentType::TEXT_HTML;

    for to_str in reply_to.iter() {
        match to_str.parse::<Mailbox>() {
            Ok(to) => {
                let email = Message::builder()
                    .from(from.clone())
                    .to(to)
                    .subject(email_subject.to_string())
                    .header(header.clone())
                    .body(body.clone())
                    .unwrap();

                match mailer.send(&email) {
                    Ok(_) => println!("✅ Email enviado a {}", to_str),
                    Err(e) => eprintln!("❌ Error al enviar a {}: {:?}", to_str, e),
                }
            }
            Err(e) => eprintln!("❌ Dirección inválida '{}': {}", to_str, e),
        }
    }
}

pub async fn send_message_email(
    recipients: Vec<(String, String)>, // (email, student_name)
    sender_name: &str,
    message: &str
) {
    use std::env;
    use lettre::message::{header::ContentType, Mailbox, Message};
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{SmtpTransport, Transport};
    use std::fs::read_to_string;
    use tera::{Context, Tera};

    let from_str = env::var("EMAIL_FROM").expect("EMAIL_FROM must be set");
    let from: Mailbox = from_str.parse().expect("Invalid EMAIL_FROM format");
    let base_dir = env::var("BASE_PATH").expect("BASE_PATH must be set");
    let template_path = format!("{}/src/email_templates/message_sent.html", base_dir);
    let template_str = match read_to_string(&template_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error cargando template: {}", e);
            return;
        }
    };
    let mut tera = Tera::default();
    tera.add_raw_template("message_sent", &template_str).expect("Template inválido");
    let email_subject = "Nuevo mensaje recibido";
    let credentials = Credentials::new(
        env::var("EMAIL_USERNAME").expect("EMAIL_USERNAME must be set"),
        env::var("EMAIL_PASSWORD").expect("EMAIL_PASSWORD must be set"),
    );
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(credentials)
        .build();
    let header = ContentType::TEXT_HTML;
    for (to_str, student_name) in recipients.iter() {
        match to_str.parse::<Mailbox>() {
            Ok(to) => {
                let mut context = Context::new();
                context.insert("sender_name", &ammonia::clean(sender_name));
                context.insert("receiver_name", &ammonia::clean(student_name));
                context.insert("message", &ammonia::clean(message));
                let body = match tera.render("message_sent", &context) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error renderizando template: {}", e);
                        continue;
                    }
                };
                let sanitized_body = ammonia::clean(&body);
                let email = Message::builder()
                    .from(from.clone())
                    .to(to)
                    .subject(email_subject.to_string())
                    .header(header.clone())
                    .body(sanitized_body)
                    .unwrap();
                match mailer.send(&email) {
                    Ok(_) => println!("✅ Email enviado a {}", to_str),
                    Err(e) => eprintln!("❌ Error al enviar a {}: {:?}", to_str, e),
                }
            }
            Err(e) => eprintln!("❌ Dirección inválida '{}': {}", to_str, e),
        }
    }
}

pub async fn send_subject_message_email(
    recipients: Vec<(String, String)>, // (email, student_name)
    sender_name: &str,
    subject_name: &str,
    message: &str
) {
    use std::env;
    use lettre::message::{header::ContentType, Mailbox, Message};
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{SmtpTransport, Transport};
    use std::fs::read_to_string;
    use tera::{Context, Tera};

    let from_str = env::var("EMAIL_FROM").expect("EMAIL_FROM must be set");
    let from: Mailbox = from_str.parse().expect("Invalid EMAIL_FROM format");
    let base_dir = env::var("BASE_PATH").expect("BASE_PATH must be set");
    let template_path = format!("{}/src/email_templates/subject_message_sent.html", base_dir);
    let template_str = match read_to_string(&template_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error cargando template: {}", e);
            return;
        }
    };
    let mut tera = Tera::default();
    tera.add_raw_template("subject_message_sent", &template_str).expect("Template inválido");
    let email_subject = format!("Nuevo mensaje en la materia: {}", subject_name);
    let credentials = Credentials::new(
        env::var("EMAIL_USERNAME").expect("EMAIL_USERNAME must be set"),
        env::var("EMAIL_PASSWORD").expect("EMAIL_PASSWORD must be set"),
    );
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(credentials)
        .build();
    let header = ContentType::TEXT_HTML;
    for (to_str, student_name) in recipients.iter() {
        match to_str.parse::<Mailbox>() {
            Ok(to) => {
                let mut context = Context::new();
                context.insert("sender_name", &ammonia::clean(sender_name));
                context.insert("receiver_name", &ammonia::clean(student_name));
                context.insert("subject_name", &ammonia::clean(subject_name));
                context.insert("message", &ammonia::clean(message));
                let body = match tera.render("subject_message_sent", &context) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error renderizando template: {}", e);
                        continue;
                    }
                };
                let sanitized_body = ammonia::clean(&body);
                let email = Message::builder()
                    .from(from.clone())
                    .to(to)
                    .subject(email_subject.clone())
                    .header(header.clone())
                    .body(sanitized_body)
                    .unwrap();
                match mailer.send(&email) {
                    Ok(_) => println!("✅ Email enviado a {}", to_str),
                    Err(e) => eprintln!("❌ Error al enviar a {}: {:?}", to_str, e),
                }
            }
            Err(e) => eprintln!("❌ Dirección inválida '{}': {}", to_str, e),
        }
    }
}

pub async fn send_assessment_email(
    recipients: Vec<(String, String, String)>, // (email, student_name, subject_name)
    sender_name: &str,
    assessment_title: &str,
    due_date: &str
) {
    use std::env;
    use lettre::message::{header::ContentType, Mailbox, Message};
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{SmtpTransport, Transport};
    use std::fs::read_to_string;
    use tera::{Context, Tera};

    let from_str = env::var("EMAIL_FROM").expect("EMAIL_FROM must be set");
    let from: Mailbox = from_str.parse().expect("Invalid EMAIL_FROM format");
    let base_dir = env::var("BASE_PATH").expect("BASE_PATH must be set");
    let template_path = format!("{}/src/email_templates/assessment_created.html", base_dir);
    let template_str = match read_to_string(&template_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error cargando template: {}", e);
            return;
        }
    };
    let mut tera = Tera::default();
    tera.add_raw_template("assessment_created", &template_str).expect("Template inválido");
    let credentials = Credentials::new(
        env::var("EMAIL_USERNAME").expect("EMAIL_USERNAME must be set"),
        env::var("EMAIL_PASSWORD").expect("EMAIL_PASSWORD must be set"),
    );
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(credentials)
        .build();
    let header = ContentType::TEXT_HTML;
    for (to_str, student_name, subject_name) in recipients.iter() {
        match to_str.parse::<Mailbox>() {
            Ok(to) => {
                let mut context = Context::new();
                context.insert("sender_name", &ammonia::clean(sender_name));
                context.insert("receiver_name", &ammonia::clean(student_name));
                context.insert("subject_name", &ammonia::clean(subject_name));
                context.insert("assessment_title", &ammonia::clean(assessment_title));
                context.insert("due_date", &ammonia::clean(due_date));
                let body = match tera.render("assessment_created", &context) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error renderizando template: {}", e);
                        continue;
                    }
                };
                let sanitized_body = ammonia::clean(&body);
                let email_subject = format!("Nueva evaluación en la materia: {}", subject_name);
                let email = Message::builder()
                    .from(from.clone())
                    .to(to)
                    .subject(email_subject)
                    .header(header.clone())
                    .body(sanitized_body)
                    .unwrap();
                match mailer.send(&email) {
                    Ok(_) => println!("✅ Email enviado a {}", to_str),
                    Err(e) => eprintln!("❌ Error al enviar a {}: {:?}", to_str, e),
                }
            }
            Err(e) => eprintln!("❌ Dirección inválida '{}': {}", to_str, e),
        }
    }
}

