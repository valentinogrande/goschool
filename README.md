
# 🎓 goschool

![Logo](https://img.icons8.com/ios-filled/100/000000/classroom.png)

**goschool** es un sistema de gestión escolar que permite administrar usuarios, cursos, materias, evaluaciones, tareas, mensajes y más, todo desde una interfaz segura y práctica.

---

## 🚀 Instalación y Base de Datos

```bash
# 🧹 Eliminar todas las tablas
python3 create_database.py delete_tables

# 🏗️ Crear todas las tablas y claves JWT
python3 create_database.py create_all
```

---

## 👥 Usuarios de prueba

| Rol        | Usuario     | Contraseña |
|------------|-------------|------------|
| Admin      | `admin`     | `admin`    |
| Estudiante | `student`   | `student`  |
| Padre      | `father`    | `father`   |
| Profesor   | `teacher`   | `teacher`  |
| Preceptor  | `preceptor` | `preceptor`|

---

## 📘 Uso de Filtros en la API

La mayoría de los endpoints de tipo GET aceptan filtros opcionales a través de la query string para refinar los resultados.
🔎 Ejemplo de endpoint

### Consulta de datos personales públicos:

```bash
 curl -X GET http://localhost:8080/api/v1/public_personal_data/ -b "jwt={jwt}"
 ```

### 🧰 Filtros disponibles (ejemplo en filters.rs)

```rust
 pub struct UserFilters {
    pub course: Option<u64>,
    pub name: Option<String>,
    pub id: Option<u64>,
} 
```

Puedes usarlos como parámetros en la URL.
✅ Ejemplos de uso de filtros

Filtrar por curso (course ID):

```bash
curl -X GET "http://localhost:8080/api/v1/public_personal_data/?course=1" -b "jwt={jwt}"
```

Filtrar por curso y nombre:

```bash
curl -X GET "http://localhost:8080/api/v1/public_personal_data/?course=1&name=pepe" -b "jwt={jwt}"
```

Filtrar por ID directamente:

```bash
curl -X GET "http://localhost:8080/api/v1/public_personal_data/?id=123" -b "jwt={jwt}"
```

## 📌 Notas

    Todos los filtros son opcionales.

    Puedes combinar varios filtros en una misma solicitud.

    Los valores deben estar URL-encoded si contienen espacios u otros caracteres especiales.

---

## 🔐 Autenticación y Roles


```bash
# 🔑 Login
curl -X POST http://localhost:8080/api/v1/login/ \
  -H "Content-Type: application/json" \
  -d '{"email": "usuario", "password": "clave", "role": "rol"}'

# 🧩 Obtener roles de un usuario
curl -X POST http://localhost:8080/api/v1/roles/ \
  -H "Content-Type: application/json" \
  -d '{"email": "usuario", "password": "clave"}'
```

---

## 🏫 Cursos y Materias

```bash
# 📖 Listar cursos
curl -X GET http://localhost:8080/api/v1/courses/ -b "jwt={jwt}"

# 📚 Listar materias
curl -X GET http://localhost:8080/api/v1/subjetcs/ -b "jwt={jwt}"
# (Nota: endpoint con typo: 'subjetcs' en lugar de 'subjects')
```

---

## 📝 Evaluaciones

```bash
# 📋 Listar evaluaciones
curl -X GET http://localhost:8080/api/v1/assessments/ -b "jwt={jwt}"

# ➕ Crear evaluación
curl -X POST http://localhost:8080/api/v1/assessments/ \
  -H "Content-Type: application/json" \
  -b "jwt={jwt}" \
  -d '{
    "newtask": {
      "subject": 1,
      "task": "nombre de la evaluación",
      "due_date": "2025-06-01",
      "type": "oral"
    }
  }'

# 🤓 Crear autoevaluable
curl -X POST http://localhost:8080/api/v1/assessments/ \
  -H "Content-Type: application/json" \
  -b "jwt={jwt}" \
  -d '{
    "newtask": {
      "subject": 1,
      "task": "autoevaluable",
      "due_date": "2025-06-01",
      "type": "selfassessable"
    },
    "newselfassessable": {
      "questions": ["Pregunta 1", "Pregunta 2"],
      "correct": ["Respuesta1", "Respuesta2"],
      "incorrect1": ["Opción1", "Opción2"],
      "incorrect2": ["Opción3", "Opción4"]
    }
  }'

# ✅ Responder autoevaluable
curl -X POST http://localhost:8080/api/v1/create_selfassessable_submission/ \
  -H "Content-Type: application/json" \
  -b "jwt={jwt}" \
  -d '{"assessment_id":1,"answers":["respuesta1","respuesta2"]}'
```

---

## 🏆 Notas

```bash
# 📊 Listar notas
curl -X GET http://localhost:8080/api/v1/grades/ -b "jwt={jwt}"

# 📝 Cargar una nota
curl -X POST http://localhost:8080/api/v1/grades/ \
  -H "Content-Type: application/json" \
  -b "jwt={jwt}" \
  -d '{
    "subject": 1,
    "assessment_id": 1,
    "student_id": 2,
    "grade_type": "numerical",
    "description": "descripcion de la nota",
    "grade": 7.5
  }'
```

---

## 🗂️ Datos personales

```bash
# 👤 Obtener datos personales
curl -X GET http://localhost:8080/api/v1/personal_data/ -b "jwt={jwt}"

# 👤 Obtener datos personales de otros (nombre y foto)
curl -X GET http://localhost:8080/api/v1/public_personal_data/ -b "jwt={jwt}"
```
---

## 📸 Foto de perfil y Archivos

```bash
# 📷 Subir foto de perfil
curl -X POST http://localhost:8080/api/v1/upload_profile_picture/ \
  -b "jwt={jwt}" -F "file=@ruta/imagen.jpg"

# 🖼️ Obtener link de la foto de perfil
curl -X GET http://localhost:8080/api/v1/get_profile_picture/ -b "jwt={jwt}"

# 📑 Subir tarea (homework)
curl -X POST http://localhost:8080/api/v1/create_submission/ \
  -H "Cookie: jwt={jwt}" \
  -F "file=@archivo.pdf" -F "homework_id=1"
```

---

## 💬 Mensajes de cada materia 


```bash 1
# ✉️ Crear mensaje

curl -X POST http://localhost:8000/api/v1/subject_messages/ \
  -H "Cookie: jwt={jwt}" \
  -F "subject_id=1" \
  -F "type=file" \
  -F "title=Material complemetario" \
  -F "content=Material complementario del módulo 3" \
  -F "file=@./documento.pdf"

# crear mensaje sin archivo 

curl -X POST http://localhost:8000/api/v1/subject_messages/ \
  -H "Cookie: jwt={jwt}" \
  -F "subject_id=1" \
  -F "title=Reunion" \
  -F "type=text" \
  -F "content=Reunión el martes a las 10hs"

# 📬 Obtener Mensajes

curl -X GET "http://localhost:8000/api/v1/subject_messages/" \
  -H "Cookie: jwt={jwt}" \
  -H "Accept: application/json"

```

## 💬 Mensajes internos

```bash
# ✉️ Crear mensaje
curl -X POST http://localhost:8080/api/v1/messages/ \
  -H "Content-Type: application/json" \
  -H "Cookie: jwt={jwt}" \
  -d '{"title":"Título","message":"Mensaje","courses":"34,35,36"}'

# 📬 Obtener mensajes
curl -X GET http://localhost:8080/api/v1/messages/ -b "jwt={jwt}"
```

---

## 🛡️ Otros Endpoints útiles

```bash
# ✔️ Verificar token JWT
curl -X GET "http://localhost:8080/api/v1/verify_token/" -b "jwt={jwt}"

# 🔍 Obtener rol actual
curl -X GET "http://localhost:8080/api/v1/get_role/" -b "jwt={jwt}"
```

---

## ℹ️ Notas importantes

- 🔄 Reemplaza `{jwt}` por el token JWT recibido tras iniciar sesión.
- 🔐 Algunos endpoints requieren un rol específico para acceder.
- 📖 Puedes consultar el código fuente para ver más detalles internos.
