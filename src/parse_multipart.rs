use actix_multipart::Multipart;
use futures_util::StreamExt;
use sanitize_filename::sanitize;
use std::collections::HashMap;
use std::env;
use std::io::Write;
use tempfile::NamedTempFile;
use uuid::Uuid;

/// Tamaño máximo permitido para subir archivos: 10 MB
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

pub async fn parse_multipart<'a>(
    mut multipart: Multipart,
    supported_extensions: Option<&[&str]>,
    supported_mime_types: Option<&[&str]>,
    upload_path: &'a str,
) -> Result<HashMap<String, Vec<u8>>, &'a str> {
    let mut fields = HashMap::new();

    // Solo permitimos un archivo por solicitud
    let mut file_received = false;

    while let Some(field_result) = multipart.next().await {
        let mut field = match field_result {
            Ok(f) => f,
            Err(_) => continue, // Ignorar errores de campo individuales
        };

        let field_name = field.name().map(String::from).unwrap_or_default();

        // Si el campo tiene filename => es archivo
        let is_file_field = field
            .content_disposition()
            .and_then(|cd| cd.get_filename())
            .is_some();

        if is_file_field {
            if file_received {
                return Err("Multiple files not allowed");
            }
            file_received = true;

            // Obtener y sanitizar el nombre del archivo original
            let filename_raw = field
                .content_disposition()
                .and_then(|cd| cd.get_filename())
                .ok_or("Missing filename")?;
            let filename_sanitized = sanitize(&filename_raw.to_lowercase());

            // Obtener extensión y validar contra permitidas
            let extension = filename_sanitized.rsplit('.').next().unwrap_or("bin");
            if let Some(exts) = supported_extensions {
                if !exts.contains(&extension) {
                    return Err("Invalid file extension");
                }
            }

            // Nombre único para almacenar (UUID + extensión)
            let unique_name = format!("{}.{}", Uuid::new_v4(), extension);
            let base_path = env::var("BASE_PATH").unwrap();
            dbg!(&base_path); // Imprime el valor de BASE_PATH
            let upload_path_str = format!("{}/{}/{}", base_path, upload_path, unique_name);
            dbg!(&upload_path_str); // Imprime la ruta completa de guardado

            // Crear archivo temporal
            let mut temp_file = NamedTempFile::new().map_err(|_| "Failed to create temp file")?;

            let mut total_size = 0usize;
            let mut content_buffer = Vec::new();

            while let Some(chunk) = field.next().await {
                let chunk = chunk.map_err(|_| "Failed to read chunk")?;
                total_size += chunk.len();
                if total_size > MAX_FILE_SIZE {
                    return Err("File too large. Limit is 10MB");
                }

                temp_file
                    .write_all(&chunk)
                    .map_err(|_| "Failed to write to temp file")?;
                if content_buffer.len() < 8192 {
                    // Solo guardar primeros 8KB para inferir MIME
                    content_buffer.extend_from_slice(&chunk);
                }
            }

            // Validar MIME type con los primeros bytes
            if let Some(mime_types) = supported_mime_types {
                let kind = infer::get(&content_buffer);
                if let Some(kind) = kind {
                    let mime = kind.mime_type();
                    if !mime_types.contains(&mime) {
                        return Err("Unsupported MIME type");
                    }
                } else {
                    return Err("Unable to detect MIME type");
                }
            }

            let path = std::path::Path::new(&upload_path_str);
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|_| "Failed to create upload directory")?;
            }

            // Mover archivo temporal a ubicación definitiva usando persist dentro de spawn_blocking
            tokio::task::spawn_blocking({
                let upload_path_str = upload_path_str.clone();
                move || {
                    temp_file
                        .persist(&upload_path_str)
                        .map_err(|_| "Failed to move uploaded file")
                }
            })
            .await
            .map_err(|_| "Blocking task failed")??;

            // Guardar en el HashMap la ruta del archivo guardado (como string en bytes)

            let url = env::var("BASE_URL").expect("BASE_URL must be set");
            let path = format!("{}{}/{}", url, upload_path, unique_name);
            dbg!(&path);
            fields.insert("file".to_string(), path.into_bytes());
        } else {
            // Campos normales (no archivo)
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                data.extend_from_slice(&chunk.map_err(|_| "Failed to read field chunk")?);
            }
            fields.insert(field_name, data);
        }
    }

    Ok(fields)
}
