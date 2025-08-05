use std::env;
use lettre::message::{header::ContentType, Mailbox, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{SmtpTransport, Transport};
use std::fs::read_to_string;
use tera::{Context, Tera};
use tokio::task;

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

    let template_path = format!("{}/email_templates/grade_submitted.html", base_dir);
    let template_str = match read_to_string(&template_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error cargando template: {}", e);
            return;
        }
    };
    let footer_template_path = format!("{}/email_templates/footer.html", base_dir);
    let footer_template_str = match read_to_string(&footer_template_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error cargando template: {}", e);
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

    let sender_name = sender_name.to_string();
    let student_name = student_name.to_string();
    let subject = subject.to_string();
    let grade = grade.to_string();

    let tasks = reply_to.into_iter().map(|to_str| {
        let from = from.clone();
        let mailer = mailer.clone();
        let header = header.clone();
        let subject_email = email_subject.to_string();
        let to_str_clone = to_str.clone();

        let template_clone = template_str.clone();
        let footer_template_clone = footer_template_str.clone();

        let sender_name = sender_name.clone();
        let student_name = student_name.clone();
        let subject = subject.clone();
        let grade = grade.clone();

        task::spawn_blocking(move || {
            match to_str_clone.parse::<Mailbox>() {
                Ok(to) => {
                    let mut context = Context::new();
                    context.insert("sender_name", &ammonia::clean(&sender_name));
                    context.insert("student_name", &ammonia::clean(&student_name));
                    context.insert("subject", &ammonia::clean(&subject));
                    context.insert("grade", &ammonia::clean(&grade));

                    let mut tera = Tera::default();
                    tera.add_raw_template("grade_submitted", &template_clone).expect("Template inválido");
                    tera.add_raw_template("footer", &footer_template_clone).expect("Template inválido");

                    let body = match tera.render("grade_submitted", &context) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Error renderizando template: {}", e);
                            return;
                        }
                    };

                    let email = Message::builder()
                        .from(from)
                        .to(to)
                        .subject(subject_email)
                        .header(header)
                        .body(body)
                        .unwrap();

                    match mailer.send(&email) {
                        Ok(_) => println!("✅ Email enviado a {}", to_str_clone),
                        Err(e) => eprintln!("❌ Error al enviar a {}: {:?}", to_str_clone, e),
                    }
                }
                Err(e) => eprintln!("❌ Dirección inválida '{}': {}", to_str_clone, e),
            }
        })
    });

    futures::future::join_all(tasks).await;
}

pub async fn send_message_email(
    recipients: Vec<(String, String)>, // (email, student_name)
    sender_name: &str,
    message: &str,
) {
    let from_str = env::var("EMAIL_FROM").expect("EMAIL_FROM must be set");
    let from: Mailbox = from_str.parse().expect("Invalid EMAIL_FROM format");
    let base_dir = env::var("BASE_PATH").expect("BASE_PATH must be set");

    let template_path = format!("{}/email_templates/message_sent.html", base_dir);
    let template_str = match read_to_string(&template_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error cargando template: {}", e);
            return;
        }
    };
    let footer_template_path = format!("{}/email_templates/footer.html", base_dir);
    let footer_template_str = match read_to_string(&footer_template_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error cargando template: {}", e);
            return;
        }
    };

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

    let tasks = recipients.into_iter().map(|(to_str, student_name)| {
        let from = from.clone();
        let mailer = mailer.clone();
        let header = header.clone();
        let subject = email_subject.to_string();
        let sender_name = sender_name.to_string();
        let message = message.to_string();
        let student_name = student_name.to_string();

        let template_clone = template_str.clone();
        let footer_template_clone = footer_template_str.clone();

        task::spawn_blocking(move || {
            match to_str.parse::<Mailbox>() {
                Ok(to) => {
                    let mut context = Context::new();
                    context.insert("sender_name", &ammonia::clean(&sender_name));
                    context.insert("receiver_name", &ammonia::clean(&student_name));
                    context.insert("message", &ammonia::clean(&message));

                    let mut tera = Tera::default();
                    tera.add_raw_template("message_sent", &template_clone).expect("Template inválido");
                    tera.add_raw_template("footer", &footer_template_clone).expect("Template inválido");


                    let body = match tera.render("message_sent", &context) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Error renderizando template: {}", e);
                            return;
                        }
                    };

                    let email = Message::builder()
                        .from(from)
                        .to(to)
                        .subject(subject)
                        .header(header)
                        .body(body)
                        .unwrap();

                    match mailer.send(&email) {
                        Ok(_) => println!("✅ Email enviado a {}", to_str),
                        Err(e) => eprintln!("❌ Error al enviar a {}: {:?}", to_str, e),
                    }
                }
                Err(e) => eprintln!("❌ Dirección inválida '{}': {}", to_str, e),
            }
        })
    });

    futures::future::join_all(tasks).await;
}

pub async fn send_subject_message_email(
    recipients: Vec<(String, String)>, // (email, student_name)
    sender_name: &str,
    subject_name: &str,
    message: &str,
) {
    let from_str = env::var("EMAIL_FROM").expect("EMAIL_FROM must be set");
    let from: Mailbox = from_str.parse().expect("Invalid EMAIL_FROM format");
    let base_dir = env::var("BASE_PATH").expect("BASE_PATH must be set");

    let template_path = format!("{}/email_templates/subject_message_sent.html", base_dir);
    let template_str = match read_to_string(&template_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error cargando template: {}", e);
            return;
        }
    };
    let footer_template_path = format!("{}/email_templates/footer.html", base_dir);
    let footer_template_str = match read_to_string(&footer_template_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error cargando template: {}", e);
            return;
        }
    };

    let credentials = Credentials::new(
        env::var("EMAIL_USERNAME").expect("EMAIL_USERNAME must be set"),
        env::var("EMAIL_PASSWORD").expect("EMAIL_PASSWORD must be set"),
    );

    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(credentials)
        .build();

    let header = ContentType::TEXT_HTML;

    let tasks = recipients.into_iter().map(|(to_str, student_name)| {
        let from = from.clone();
        let mailer = mailer.clone();
        let header = header.clone();
        let sender_name = sender_name.to_string();
        let subject_name = subject_name.to_string();
        let message = message.to_string();

        let template_clone = template_str.clone();
        let footer_template_clone = footer_template_str.clone();

        task::spawn_blocking(move || {
            match to_str.parse::<Mailbox>() {
                Ok(to) => {
                    let mut context = Context::new();
                    context.insert("sender_name", &ammonia::clean(&sender_name));
                    context.insert("receiver_name", &ammonia::clean(&student_name));
                    context.insert("subject_name", &ammonia::clean(&subject_name));
                    context.insert("message", &ammonia::clean(&message));

                    let mut tera = Tera::default();
                    tera.add_raw_template("subject_message_sent", &template_clone).expect("Template inválido");
                    tera.add_raw_template("footer", &footer_template_clone).expect("Template inválido");

                    let body = match tera.render("subject_message_sent", &context) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Error renderizando template: {}", e);
                            return;
                        }
                    };

                    let email_subject = format!("Nuevo mensaje en la materia: {}", subject_name);

                    let email = Message::builder()
                        .from(from)
                        .to(to)
                        .subject(email_subject)
                        .header(header)
                        .body(body)
                        .unwrap();

                    match mailer.send(&email) {
                        Ok(_) => println!("✅ Email enviado a {}", to_str),
                        Err(e) => eprintln!("❌ Error al enviar a {}: {:?}", to_str, e),
                    }
                }
                Err(e) => eprintln!("❌ Dirección inválida '{}': {}", to_str, e),
            }
        })
    });

    futures::future::join_all(tasks).await;
}

pub async fn send_assessment_email(
    recipients: Vec<(String, String, String)>, // (email, student_name, subject_name)
    sender_name: &str,
    assessment_title: &str,
    due_date: &str,
) {
    let from_str = env::var("EMAIL_FROM").expect("EMAIL_FROM must be set");
    let from: Mailbox = from_str.parse().expect("Invalid EMAIL_FROM format");
    let base_dir = env::var("BASE_PATH").expect("BASE_PATH must be set");

    let template_path = format!("{}/email_templates/assessment_created.html", base_dir);
    let template_str = match read_to_string(&template_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error cargando template: {}", e);
            return;
        }
    };
    let footer_template_path = format!("{}/email_templates/footer.html", base_dir);
    let footer_template_str = match read_to_string(&footer_template_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error cargando template: {}", e);
            return;
        }
    };

    let credentials = Credentials::new(
        env::var("EMAIL_USERNAME").expect("EMAIL_USERNAME must be set"),
        env::var("EMAIL_PASSWORD").expect("EMAIL_PASSWORD must be set"),
    );

    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(credentials)
        .build();

    let header = ContentType::TEXT_HTML;

    let tasks = recipients.into_iter().map(|(to_str, student_name, subject_name)| {
        let from = from.clone();
        let mailer = mailer.clone();
        let header = header.clone();
        let sender_name = sender_name.to_string();
        let assessment_title = assessment_title.to_string();
        let due_date = due_date.to_string();
        let student_name = student_name.to_string();
        let subject_name = subject_name.to_string();

        let template_clone = template_str.clone();
        let footer_template_clone = footer_template_str.clone();

        task::spawn_blocking(move || {
            match to_str.parse::<Mailbox>() {
                Ok(to) => {
                    let mut context = Context::new();
                    context.insert("sender_name", &ammonia::clean(&sender_name));
                    context.insert("receiver_name", &ammonia::clean(&student_name));
                    context.insert("subject_name", &ammonia::clean(&subject_name));
                    context.insert("assessment_title", &ammonia::clean(&assessment_title));
                    context.insert("due_date", &ammonia::clean(&due_date));

                    let mut tera = Tera::default();
                    tera.add_raw_template("assessment_created", &template_clone).expect("Template inválido");
                    tera.add_raw_template("footer", &footer_template_clone).expect("Template inválido");

                    let body = match tera.render("assessment_created", &context) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Error renderizando template: {}", e);
                            return;
                        }
                    };

                    let email_subject = format!("Nueva evaluación en la materia: {}", subject_name);

                    let email = Message::builder()
                        .from(from)
                        .to(to)
                        .subject(email_subject)
                        .header(header)
                        .body(body)
                        .unwrap();

                    match mailer.send(&email) {
                        Ok(_) => println!("✅ Email enviado a {}", to_str),
                        Err(e) => eprintln!("❌ Error al enviar a {}: {:?}", to_str, e),
                    }
                }
                Err(e) => eprintln!("❌ Dirección inválida '{}': {}", to_str, e),
            }
        })
    });

    futures::future::join_all(tasks).await;
}

