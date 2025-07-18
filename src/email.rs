use std::env;
use lettre::message::{header::ContentType, Mailbox, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{SmtpTransport, Transport};
use std::fs::read_to_string;
use tera::{Context, Tera};


#[allow(dead_code)]
pub async fn send_email(reply_to: Vec<String>, subject: String) {
    
    let from_str = env::var("EMAIL_FROM").expect("EMAIL_FROM must be set");
    let from: Mailbox = from_str.parse().expect("Invalid EMAIL_FROM format");

    let header = ContentType::TEXT_HTML;

    let body = render_email("GoSchool", "Tino", &subject, "10");

    let credentials = Credentials::new(
        env::var("EMAIL_USERNAME").expect("EMAIL_USERNAME must be set"),
        env::var("EMAIL_PASSWORD").expect("EMAIL_PASSWORD must be set"),
    );

    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(credentials)
        .build();

    for to_str in reply_to.iter() {
        match to_str.parse::<Mailbox>() {
            Ok(to) => {
                let email = Message::builder()
                    .from(from.clone())
                    .to(to)
                    .subject(subject.clone())
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

fn render_email(sender_name: &str, student_name: &str, subject: &str, grade: &str) -> String {
    let mut tera = Tera::default();

    let template_path = {
        match subject {
        "Grade submitted" => "email_templates/grade_submitted.html",
        _ => "email_templates/no_valid_subject.html",
        }
    };

    let template_str = read_to_string(template_path)
        .unwrap_or_else(|_| "<p>Error cargando template</p>".to_string());

    tera.add_raw_template("grade_submitted", &template_str).expect("Template inválido");

    let mut context = Context::new();
    context.insert("sender_name", sender_name);
    context.insert("student_name", student_name);
    context.insert("subject", subject);
    context.insert("grade", grade);

    tera.render("grade_submitted", &context)
        .unwrap_or_else(|e| format!("Error renderizando template: {}", e))
}
