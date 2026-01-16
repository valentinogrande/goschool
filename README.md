# 🎓 GoSchool API Documentation

![Logo](https://img.icons8.com/ios-filled/100/000000/classroom.png)

**GoSchool** is a comprehensive school management system API built with Rust and Actix-web that allows you to manage users, courses, subjects, assessments, assignments, messages, and more through a secure and practical interface.

## 📋 Table of Contents

- [Project Architecture](#-project-architecture)
- [Installation & Setup](#-installation--setup)
- [Authentication](#-authentication)
- [API Overview](#-api-overview)
- [REST API Endpoints](#-endpoints-documentation)
  - [Authentication & Authorization](#authentication--authorization)
  - [Users & Personal Data](#users--personal-data)
  - [Academic Management](#academic-management)
  - [Assessments & Grades](#assessments--grades)
  - [Messages & Communications](#messages--communications)
  - [File Management](#file-management)
  - [Attendance & Discipline](#attendance--discipline)
  - [Utility Endpoints](#utility-endpoints)
- [Chat System](#-chat-system)
  - [Chat REST API](#chat-rest-api)
  - [WebSocket API](#websocket-api)
- [Data Models](#-data-models)
- [Filtering System](#-filtering-system)
- [Error Handling](#-error-handling)
- [Security Features](#-security-features)
- [Database Schema](#-database-schema)
- [Development Guide](#-development-guide)

---

## 🏗️ Project Architecture

### Technology Stack

| Component | Technology | Version |
|-----------|------------|---------|
| Web Framework | Actix-web | 4.x |
| Async Runtime | Tokio | 1.44.2 |
| Database | MySQL + SQLx | 8.0 / 0.8 |
| Authentication | JWT (ES256) | jsonwebtoken 9.3 |
| Password Hashing | bcrypt | 0.17 |
| WebSocket | actix-ws | 0.3 |
| Email | Lettre + Tera | 0.11 / 1.x |
| Scheduling | Cron | 0.15 |

### Directory Structure

```
back/
├── src/
│   ├── main.rs              # Server setup, middleware, static files
│   ├── routes.rs            # Route registration & service config
│   ├── structs.rs           # Data models (User, Grade, Assessment, etc.)
│   ├── traits.rs            # CRUD trait definitions
│   ├── filters.rs           # Query filter definitions
│   ├── jwt.rs               # JWT creation & validation
│   ├── json.rs              # JSON parsing configuration
│   ├── cron.rs              # Scheduled tasks (auto-grading)
│   ├── email.rs             # Email service with templates
│   ├── parse_multipart.rs   # File upload handling
│   ├── views/               # API endpoint handlers (27 files)
│   │   ├── login.rs         # Authentication endpoints
│   │   ├── grades.rs        # Grade management
│   │   ├── assessments.rs   # Assessment/task management
│   │   ├── chats.rs         # Chat REST endpoints
│   │   └── ...              # Other resource handlers
│   ├── impls/               # Trait implementations
│   │   ├── user_get.rs      # GET operations
│   │   ├── user_post.rs     # POST operations
│   │   ├── user_update.rs   # PUT operations
│   │   ├── user_delete.rs   # DELETE operations
│   │   └── chat_authorization.rs
│   └── websocket/           # Real-time chat
│       ├── handler.rs       # WebSocket connection handling
│       ├── manager.rs       # Connection state management
│       ├── protocol.rs      # Message type definitions
│       └── mod.rs
├── Cargo.toml               # Dependencies
├── init.sql                 # Database schema
├── create_database.py       # Setup script
├── fake_data.py             # Test data generator
└── email_templates/         # HTML email templates
```

### Request Flow

```
Client Request
     │
     ▼
┌─────────────────┐
│  Nginx Proxy    │  (SSL termination, rate limiting)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Actix-web      │  (Routing, CORS, Logging)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  JWT Middleware │  (Token validation, role extraction)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  View Handler   │  (Request processing)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Trait Impl     │  (Database operations via SQLx)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  MySQL          │  (Data persistence)
└─────────────────┘
```

---

## 🚀 Installation & Setup

### Prerequisites
- Rust and Cargo
- MySQL database
- Required environment variables

### Database Setup

```bash
# 🧹 Delete all tables
python3 create_database.py delete_tables

# 🏗️ Create all tables and JWT keys
python3 create_database.py create_all
```

### Test Users

| Role        | Username    | Password   |
|-------------|-------------|------------|
| Admin       | `admin`     | `admin`    |
| Student     | `student`   | `student`  |
| Parent      | `father`    | `father`   |
| Teacher     | `teacher`   | `teacher`  |
| Preceptor   | `preceptor` | `preceptor`|

---

## 🔐 Authentication

GoSchool uses **JWT (JSON Web Tokens)** for authentication with **ES256** algorithm and role-based access control.

### Authentication Flow
1. Login with credentials and role
2. Receive JWT token as HTTP-only cookie
3. Include cookie in subsequent requests
4. Token expires after 1 hour

### Supported Roles
- **admin**: Full system access
- **teacher**: Academic management for assigned subjects
- **student**: Access to own academic data
- **preceptor**: Student management and discipline
- **father**: Access to children's academic information

---

## 📡 API Overview

- **Base URL**: `http://localhost:80/api/v1/`
- **Authentication**: JWT Cookie (`jwt`)
- **Content-Type**: `application/json` (for JSON endpoints)
- **File Uploads**: `multipart/form-data`
- **Date Format**: ISO 8601 (`YYYY-MM-DD`)
- **Time Zone**: UTC for all timestamps

---

## 📚 Endpoints Documentation

### Authentication & Authorization

#### `POST /api/v1/login/`
Authenticate user with credentials and role.

**Request Body:**
```json
{
  "email": "admin",
  "password": "admin", 
  "role": "admin"
}
```

**Response:** `200 OK`
```json
"login success"
```

**Example:**
```bash
curl -X POST http://localhost:80/api/v1/login/ \
  -H "Content-Type: application/json" \
  -d '{"email": "admin", "password": "admin", "role": "admin"}'
```

---

#### `POST /api/v1/logout/`
Logout current user by invalidating JWT cookie.

**Response:** `200 OK`
```json
"logout success"
```

**Example:**
```bash
curl -X POST http://localhost:80/api/v1/logout/
```

---

#### `POST /api/v1/roles/`
Get all available roles for a user.

**Request Body:**
```json
{
  "email": "admin",
  "password": "admin"
}
```

**Response:** `200 OK`
```json
["admin", "teacher"]
```

**Example:**
```bash
curl -X POST http://localhost:80/api/v1/roles/ \
  -H "Content-Type: application/json" \
  -d '{"email": "admin", "password": "admin"}'
```

---

#### `GET /api/v1/role/`
Get current user's active role.

**Response:** `200 OK`
```json
"admin"
```

**Example:**
```bash
curl -X GET http://localhost:80/api/v1/role/ -b "jwt={jwt}"
```

---

#### `GET /api/v1/verify_token/`
Verify if current JWT token is valid.

**Response:** `200 OK`
```json
"json web token is valid"
```

**Example:**
```bash
curl -X GET http://localhost:80/api/v1/verify_token/ -b "jwt={jwt}"
```

---

### Users & Personal Data

#### `POST /api/v1/register/`
Register a new user (Admin only in production).

**Request Body:**
```json
{
  "email": "newuser@example.com",
  "password": "securepassword",
  "role": "student"
}
```

**Response:** `201 Created`

**Example:**
```bash
curl -X POST http://localhost:80/api/v1/register/ \
  -H "Content-Type: application/json" \
  -b "jwt={jwt}" \
  -d '{"email": "student@example.com", "password": "pass123", "role": "student"}'
```

---

#### `GET /api/v1/register_testing_users/`
Create default test users for development.

**Response:** `201 Created`

**Example:**
```bash
curl -X GET http://localhost:80/api/v1/register_testing_users/
```

---

#### `GET /api/v1/personal_data/`
Get current user's personal information.

**Response:** `200 OK`
```json
{
  "full_name": "John Doe",
  "phone_number": "123-456-7890",
  "address": "123 Main St",
  "birth_date": "1990-01-15"
}
```

**Example:**
```bash
curl -X GET http://localhost:80/api/v1/personal_data/ -b "jwt={jwt}"
```

---

#### `GET /api/v1/public_personal_data/`
Get public personal data (name and photo) of users.

**Query Parameters:**
- `course` (optional): Filter by course ID
- `name` (optional): Filter by name
- `user_id` (optional): Filter by user ID

**Response:** `200 OK`
```json
[
  {
    "full_name": "John Doe",
    "photo": "http://localhost:80/uploads/profile_pictures/uuid.jpg"
  }
]
```

**Example:**
```bash
curl -X GET "http://localhost:80/api/v1/public_personal_data/?course=1" -b "jwt={jwt}"
```

---

#### `PUT /api/v1/personal_data/{user_id}`
Update personal data for a specific user.

**Request Body:**
```json
{
  "full_name": "Updated Name",
  "phone_number": "+1234567890",
  "address": "New Address 123",
  "birth_date": "1990-01-15"
}
```

**Response:** `200 OK`

---

#### `DELETE /api/v1/personal_data/{user_id}`
Delete personal data for a specific user.

**Response:** `200 OK`

---

#### `GET /api/v1/students/`
Get list of student IDs with optional filtering.

**Query Parameters:**
- `course` (optional): Filter by course ID
- `name` (optional): Filter by name
- `user_id` (optional): Filter by user ID

**Response:** `200 OK`
```json
[1, 2, 3, 4]
```

**Example:**
```bash
curl -X GET "http://localhost:80/api/v1/students/?course=1" -b "jwt={jwt}"
```

---

### Academic Management

#### `GET /api/v1/courses/`
Get list of all courses.

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "year": 2025,
    "division": "A",
    "level": "1st",
    "shift": "morning",
    "preceptor_id": 5,
    "name": "1st A - Morning"
  }
]
```

**Example:**
```bash
curl -X GET http://localhost:80/api/v1/courses/ -b "jwt={jwt}"
```

---

#### `GET /api/v1/subjects/`
Get list of subjects with filtering options.

**Query Parameters:**
- `teacher_id` (optional): Filter by teacher ID
- `course_id` (optional): Filter by course ID
- `name` (optional): Filter by subject name
- `subject_id` (optional): Filter by subject ID

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "name": "Mathematics",
    "teacher_id": 2,
    "course_id": 1,
    "course_name": "1st A - Morning"
  }
]
```

**Example:**
```bash
curl -X GET "http://localhost:80/api/v1/subjects/?course_id=1" -b "jwt={jwt}"
```

---

#### `GET /api/v1/timetables/`
Get class schedules/timetables.

**Query Parameters:**
- `teacher_id` (optional): Filter by teacher ID
- `course_id` (optional): Filter by course ID
- `subject_id` (optional): Filter by subject ID
- `day` (optional): Filter by day of week

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "course_id": 1,
    "subject_id": 1,
    "day": "Monday",
    "start_time": "08:00:00",
    "end_time": "09:30:00"
  }
]
```

**Example:**
```bash
curl -X GET "http://localhost:80/api/v1/timetables/?course_id=1" -b "jwt={jwt}"
```

---

### Assessments & Grades

#### `GET /api/v1/assessments/`
Get list of assessments/tasks.

**Query Parameters:**
- `subject_id` (optional): Filter by subject
- `task` (optional): Filter by task name
- `due` (optional): Filter by due status

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "subject_id": 1,
    "task": "Math Quiz Chapter 1",
    "due_date": "2025-12-01",
    "created_at": "2025-01-15T10:30:00Z",
    "type": "exam"
  }
]
```

**Example:**
```bash
curl -X GET "http://localhost:80/api/v1/assessments/?subject_id=1" -b "jwt={jwt}"
```

---

#### `POST /api/v1/assessments/`
Create a new assessment (exam, homework, project, etc.).

**Request Body for Regular Assessment:**
```json
{
  "newtask": {
    "subject": 1,
    "task": "Mathematics Exam",
    "due_date": "2025-06-01",
    "type": "exam"
  }
}
```

**Request Body for Self-Assessment:**
```json
{
  "newtask": {
    "subject": 1,
    "task": "Self Assessment Quiz",
    "due_date": "2025-06-01",
    "type": "selfassessable"
  },
  "newselfassessable": {
    "questions": ["What is 2+2?", "What is 3+3?"],
    "correct": ["4", "6"],
    "incorrect1": ["3", "5"],
    "incorrect2": ["5", "7"]
  }
}
```

**Assessment Types:**
- `exam`: Traditional examination
- `homework`: Take-home assignment
- `project`: Long-term project
- `oral`: Oral examination
- `remedial`: Remedial assessment
- `selfassessable`: Auto-graded quiz

**Response:** `201 Created`

**Examples:**
```bash
# Create regular assessment
curl -X POST http://localhost:80/api/v1/assessments/ \
  -H "Content-Type: application/json" \
  -b "jwt={jwt}" \
  -d '{
    "newtask": {
      "subject": 1,
      "task": "Final Exam",
      "due_date": "2025-06-01",
      "type": "exam"
    }
  }'

# Create self-assessable quiz
curl -X POST http://localhost:80/api/v1/assessments/ \
  -H "Content-Type: application/json" \
  -b "jwt={jwt}" \
  -d '{
    "newtask": {
      "subject": 1,
      "task": "Quiz Chapter 1",
      "due_date": "2025-06-01",
      "type": "selfassessable"
    },
    "newselfassessable": {
      "questions": ["Question 1?", "Question 2?"],
      "correct": ["Answer1", "Answer2"],
      "incorrect1": ["Wrong1", "Wrong2"],
      "incorrect2": ["Wrong3", "Wrong4"]
    }
  }'
```

---

#### `PUT /api/v1/assessments/{id}`
Update an existing assessment.

**Request Body:**
```json
{
  "subject_id": 2,
  "task": "Updated Assessment Name",
  "due_date": "2025-07-01",
  "type_": "homework"
}
```

**Response:** `200 OK`

---

#### `DELETE /api/v1/assessments/{id}`
Delete an assessment.

**Response:** `200 OK`

---

#### `GET /api/v1/selfassessables/`
Get self-assessment questions for students.

**Query Parameters:**
- `assessment_id` (optional): Filter by assessment ID

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "question": "What is 2+2?",
    "op1": "3",
    "op2": "4",
    "op3": "5",
    "op4": null,
    "op5": null
  }
]
```

**Example:**
```bash
curl -X GET "http://localhost:80/api/v1/selfassessables/?assessment_id=1" -b "jwt={jwt}"
```

---

#### `POST /api/v1/selfassessables/`
Submit answers for a self-assessment.

**Request Body:**
```json
{
  "assessment_id": 1,
  "answers": ["4", "6", "8"]
}
```

**Response:** `200 OK`

**Example:**
```bash
curl -X POST http://localhost:80/api/v1/selfassessables/ \
  -H "Content-Type: application/json" \
  -b "jwt={jwt}" \
  -d '{"assessment_id":1,"answers":["answer1","answer2"]}'
```

---

#### `GET /api/v1/selfassessables_responses/`
Get self-assessment responses (for teachers/admins).

**Query Parameters:**
- `assessment_id` (optional): Filter by assessment ID

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "selfassessable_id": 1,
    "answers": "answer1,answer2",
    "student_id": 3
  }
]
```

---

#### `GET /api/v1/grades/`
Get student grades.

**Query Parameters:**
- `subject_id` (optional): Filter by subject
- `student_id` (optional): Filter by student
- `description` (optional): Filter by description

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "description": "Midterm Exam",
    "grade": 8.5,
    "student_id": 3,
    "subject_id": 1,
    "assessment_id": 1,
    "grade_type": "numerical",
    "created_at": "2025-01-15T10:30:00Z"
  }
]
```

**Example:**
```bash
curl -X GET "http://localhost:80/api/v1/grades/?student_id=3" -b "jwt={jwt}"
```

---

#### `POST /api/v1/grades/`
Create a new grade entry.

**Request Body:**
```json
{
  "subject": 1,
  "assessment_id": 1,
  "student_id": 3,
  "grade_type": "numerical",
  "description": "Final Exam Grade",
  "grade": 9.2
}
```

**Grade Types:**
- `numerical`: Numeric grade (0-10)
- `conceptual`: Conceptual grade (A, B, C, etc.)
- `percentage`: Percentage grade (0-100%)

**Response:** `201 Created`

**Example:**
```bash
curl -X POST http://localhost:80/api/v1/grades/ \
  -H "Content-Type: application/json" \
  -b "jwt={jwt}" \
  -d '{
    "subject": 1,
    "assessment_id": 1,
    "student_id": 2,
    "grade_type": "numerical",
    "description": "Final exam result",
    "grade": 8.5
  }'
```

---

#### `PUT /api/v1/grades/{id}`
Update an existing grade.

**Request Body:**
```json
{
  "grade": 9.5,
  "description": "Updated description"
}
```

**Response:** `200 OK`

---

#### `DELETE /api/v1/grades/{id}`
Delete a grade entry.

**Response:** `200 OK`

---

### Messages & Communications

#### `GET /api/v1/messages/`
Get internal system messages.

**Query Parameters:**
- `sender_id` (optional): Filter by sender
- `title` (optional): Filter by title
- `course_id` (optional): Filter by course

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "title": "Important Announcement",
    "message": "School will be closed tomorrow",
    "sender_id": 1,
    "created_at": "2025-01-15T10:30:00Z"
  }
]
```

**Example:**
```bash
curl -X GET http://localhost:80/api/v1/messages/ -b "jwt={jwt}"
```

---

#### `POST /api/v1/messages/`
Create a new internal message.

**Request Body:**
```json
{
  "title": "Announcement Title",
  "message": "Message content here",
  "courses": "1,2,3"
}
```

**Response:** `201 Created`

**Example:**
```bash
curl -X POST http://localhost:80/api/v1/messages/ \
  -H "Content-Type: application/json" \
  -b "jwt={jwt}" \
  -d '{"title":"Meeting","message":"Staff meeting tomorrow","courses":"1,2,3"}'
```

---

#### `PUT /api/v1/messages/{id}`
Update an existing message.

**Request Body:**
```json
{
  "title": "Updated Title",
  "message": "Updated message content"
}
```

**Response:** `200 OK`

---

#### `DELETE /api/v1/messages/{id}`
Delete a message.

**Response:** `200 OK`

---

#### `GET /api/v1/subject_messages/`
Get subject-specific messages (announcements, materials, links).

**Query Parameters:**
- `subject_message_id` (optional): Filter by message ID
- `sender_id` (optional): Filter by sender
- `subject_id` (optional): Filter by subject

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "sender_id": 2,
    "subject_id": 1,
    "title": "Chapter 1 Materials",
    "content": "Please review the attached materials",
    "created_at": "2025-01-15T10:30:00Z",
    "type": "file"
  }
]
```

**Example:**
```bash
curl -X GET "http://localhost:80/api/v1/subject_messages/?subject_id=1" -b "jwt={jwt}"
```

---

#### `POST /api/v1/subject_messages/`
Create a subject message with optional file attachment.

**Form Data:**
- `subject_id`: Subject ID (required)
- `title`: Message title (required)
- `content`: Message content (required)
- `type`: Message type - `message`, `link`, or `file` (required)
- `file`: File attachment (optional, for type=file)

**Message Types:**
- `message`: Text-only message
- `link`: Message with external link
- `file`: Message with file attachment

**Response:** `201 Created`

**Examples:**
```bash
# Create message with file
curl -X POST http://localhost:80/api/v1/subject_messages/ \
  -b "jwt={jwt}" \
  -F "subject_id=1" \
  -F "type=file" \
  -F "title=Study Materials" \
  -F "content=Chapter 3 supplementary materials" \
  -F "file=@./document.pdf"

# Create text message
curl -X POST http://localhost:80/api/v1/subject_messages/ \
  -b "jwt={jwt}" \
  -F "subject_id=1" \
  -F "title=Class Reminder" \
  -F "type=message" \
  -F "content=Remember the test on Tuesday"
```

---

#### `PUT /api/v1/subject_messages/{id}`
Update a subject message.

**Request Body:**
```json
{
  "title": "Updated Title",
  "content": "Updated content",
  "type_": "message"
}
```

**Response:** `200 OK`

---

#### `DELETE /api/v1/subject_messages/{id}`
Delete a subject message.

**Response:** `200 OK`

---

### File Management

#### `GET /api/v1/profile_pictures/`
Get current user's profile picture URL.

**Response:** `200 OK`
```json
{
  "url": "http://localhost:80/uploads/profile_pictures/uuid.jpg"
}
```

**Example:**
```bash
curl -X GET http://localhost:80/api/v1/profile_pictures/ -b "jwt={jwt}"
```

---

#### `POST /api/v1/profile_pictures/`
Upload profile picture for current user.

**Form Data:**
- `file`: Image file (JPEG, PNG, etc.)

**File Limitations:**
- Maximum size: 10MB
- Supported formats: Images (automatically detected via MIME type)

**Response:** `201 Created`

**Example:**
```bash
curl -X POST http://localhost:80/api/v1/profile_pictures/ \
  -b "jwt={jwt}" \
  -F "file=@profile.jpg"
```

---

#### `PUT /api/v1/profile_pictures/{user_id}`
Update profile picture for specific user.

**Form Data:**
- `file`: Image file

**Response:** `200 OK`

---

#### `DELETE /api/v1/profile_pictures/{user_id}`
Delete profile picture for specific user.

**Response:** `200 OK`

---

#### `POST /api/v1/homework_submission/`
Submit homework assignment file.

**Form Data:**
- `homework_id`: Assessment ID (required)
- `file`: Assignment file (required)

**Response:** `201 Created`

**Example:**
```bash
curl -X POST http://localhost:80/api/v1/homework_submission/ \
  -b "jwt={jwt}" \
  -F "file=@assignment.pdf" \
  -F "homework_id=1"
```

---

#### `PUT /api/v1/homework_submission/{id}`
Update homework submission.

**Request Body:**
```json
{
  "path": "new/file/path.pdf",
  "student_id": 3,
  "task_id": 1
}
```

**Response:** `200 OK`

---

#### `DELETE /api/v1/homework_submission/{id}`
Delete homework submission.

**Response:** `200 OK`

---

### Attendance & Discipline

#### `GET /api/v1/assistance/`
Get attendance records.

**Query Parameters:**
- `assistance_id` (optional): Filter by attendance ID
- `student_id` (optional): Filter by student
- `presence` (optional): Filter by presence status
- `date` (optional): Filter by date (YYYY-MM-DD)

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "student_id": 3,
    "presence": "present",
    "date": "2025-01-15"
  }
]
```

**Presence Status Values:**
- `present`: Student was present
- `absent`: Student was absent
- `late`: Student arrived late
- `justified`: Absence was justified

**Example:**
```bash
curl -X GET "http://localhost:80/api/v1/assistance/?student_id=3" -b "jwt={jwt}"
```

---

#### `POST /api/v1/assistance/`
Create attendance record.

**Request Body:**
```json
{
  "student_id": 3,
  "presence": "present",
  "date": "2025-01-15"
}
```

**Response:** `201 Created`

**Example:**
```bash
curl -X POST http://localhost:80/api/v1/assistance/ \
  -H "Content-Type: application/json" \
  -b "jwt={jwt}" \
  -d '{"student_id": 3, "presence": "present", "date": "2025-01-15"}'
```

---

#### `PUT /api/v1/assistance/{id}`
Update attendance record.

**Request Body:**
```json
{
  "student_id": 3,
  "presence": "late",
  "date": "2025-01-15"
}
```

**Response:** `200 OK`

---

#### `DELETE /api/v1/assistance/{id}`
Delete attendance record.

**Response:** `200 OK`

---

#### `GET /api/v1/disciplinary_sanction/`
Get disciplinary sanctions.

**Query Parameters:**
- `disciplinary_sanction_id` (optional): Filter by sanction ID
- `student_id` (optional): Filter by student
- `sanction_type` (optional): Filter by sanction type

**Response:** `200 OK`
```json
[
  {
    "student_id": 3,
    "sanction_type": "warning",
    "quantity": 1,
    "description": "Late to class",
    "date": "2025-01-15"
  }
]
```

**Common Sanction Types:**
- `warning`: Verbal or written warning
- `detention`: After-school detention
- `suspension`: Temporary suspension
- `expulsion`: Permanent removal

**Example:**
```bash
curl -X GET "http://localhost:80/api/v1/disciplinary_sanction/?student_id=3" -b "jwt={jwt}"
```

---

#### `POST /api/v1/disciplinary_sanction/`
Create disciplinary sanction.

**Request Body:**
```json
{
  "student_id": 3,
  "sanction_type": "warning",
  "quantity": 1,
  "description": "Disrupting class",
  "date": "2025-01-15"
}
```

**Response:** `201 Created`

**Example:**
```bash
curl -X POST http://localhost:80/api/v1/disciplinary_sanction/ \
  -H "Content-Type: application/json" \
  -b "jwt={jwt}" \
  -d '{
    "student_id": 3,
    "sanction_type": "warning", 
    "quantity": 1,
    "description": "Late arrival",
    "date": "2025-01-15"
  }'
```

---

#### `PUT /api/v1/disciplinary_sanction/{id}`
Update disciplinary sanction.

**Request Body:**
```json
{
  "description": "Updated description",
  "quantity": 2
}
```

**Response:** `200 OK`

---

#### `DELETE /api/v1/disciplinary_sanction/{id}`
Delete disciplinary sanction.

**Response:** `200 OK`

---

### Utility Endpoints

#### `POST /api/v1/get_if_homework_answered/`
Check if homework has been submitted by current user.

**Request Body:**
```json
{
  "homework_id": 2
}
```

**Response:** `200 OK`
```json
true
```

**Example:**
```bash
curl -X POST http://localhost:80/api/v1/get_if_homework_answered/ \
  -H "Content-Type: application/json" \
  -b "jwt={jwt}" \
  -d '{"homework_id": 2}'
```

---

#### `POST /api/v1/get_if_selfassessable_answered/`
Check if self-assessment has been completed by current user.

**Request Body:**
```json
{
  "selfassessable_id": 2
}
```

**Response:** `200 OK`
```json
false
```

**Example:**
```bash
curl -X POST http://localhost:80/api/v1/get_if_selfassessable_answered/ \
  -H "Content-Type: application/json" \
  -b "jwt={jwt}" \
  -d '{"selfassessable_id": 2}'
```

---

#### `GET /api/v1/health/`
Health check endpoint.

**Response:** `200 OK`
```
OK
```

**Example:**
```bash
curl -X GET http://localhost:80/api/v1/health/
```

---

## 📊 Data Models

### User Roles
```rust
enum Role {
    admin,      // Full system access
    teacher,    // Academic management
    student,    // Student portal access  
    preceptor,  // Student management
    father,     // Parent portal access
}
```

### Assessment Types
```rust
enum AssessmentType {
    exam,           // Traditional exam
    homework,       // Take-home assignment
    project,        // Long-term project
    oral,           // Oral examination
    remedial,       // Remedial assessment
    selfassessable, // Auto-graded quiz
}
```

### Grade Types
```rust
enum GradeType {
    numerical,   // Numeric grade (0-10)
    conceptual,  // Letter grade (A, B, C)
    percentage,  // Percentage (0-100%)
}
```

### Subject Message Types
```rust
enum SubjectMessageType {
    message, // Text message
    link,    // External link
    file,    // File attachment
}
```

---

## 🔍 Filtering System

Most GET endpoints support optional query parameters for filtering results:

### Common Filters

#### User Filters
- `course`: Filter by course ID
- `name`: Filter by name (partial match)  
- `user_id`: Filter by specific user ID

#### Academic Filters
- `subject_id`: Filter by subject ID
- `teacher_id`: Filter by teacher ID
- `course_id`: Filter by course ID

#### Assessment Filters
- `subject_id`: Filter by subject
- `task`: Filter by task name (partial match)
- `due`: Filter by due status (boolean)

#### Grade Filters
- `subject_id`: Filter by subject
- `student_id`: Filter by student
- `description`: Filter by description (partial match)

#### Message Filters
- `sender_id`: Filter by sender
- `title`: Filter by title (partial match)
- `course_id`: Filter by target course

#### Timetable Filters
- `teacher_id`: Filter by teacher
- `course_id`: Filter by course
- `subject_id`: Filter by subject
- `day`: Filter by day of week

#### Attendance Filters
- `assistance_id`: Filter by attendance record ID
- `student_id`: Filter by student
- `presence`: Filter by presence status
- `date`: Filter by specific date (YYYY-MM-DD)

#### Discipline Filters
- `disciplinary_sanction_id`: Filter by sanction ID
- `student_id`: Filter by student
- `sanction_type`: Filter by sanction type

### Filter Usage Examples

```bash
# Multiple filters
curl -X GET "http://localhost:80/api/v1/public_personal_data/?course=1&name=john" -b "jwt={jwt}"

# Date filtering
curl -X GET "http://localhost:80/api/v1/assistance/?date=2025-01-15&student_id=3" -b "jwt={jwt}"

# Subject and teacher filtering
curl -X GET "http://localhost:80/api/v1/subjects/?teacher_id=2&course_id=1" -b "jwt={jwt}"
```

**Note**: All filters are optional and can be combined. Values should be URL-encoded if they contain special characters.

---

## 💬 Chat System

GoSchool includes a real-time chat system supporting direct messages and group conversations.

### Chat REST API

#### `GET /api/v1/chats/`
Get list of user's chats with metadata.

**Query Parameters:**
- `name` (optional): Filter by chat name

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "name": "Math Study Group",
    "photo": "http://localhost:80/uploads/chat_files/uuid.jpg",
    "description": "Group for math discussions",
    "created_at": "2025-01-15T10:30:00Z",
    "updated_at": "2025-01-15T14:00:00Z",
    "chat_type": "group",
    "created_by": 2,
    "unread_count": 5,
    "last_message": {
      "message": "Hello everyone!",
      "sender_name": "John Doe",
      "created_at": "2025-01-15T14:00:00Z"
    }
  }
]
```

**Example:**
```bash
curl -X GET http://localhost:80/api/v1/chats/ -b "jwt={jwt}"
```

---

#### `POST /api/v1/chats/`
Create a new direct or group chat.

**Request Body (Direct Chat):**
```json
{
  "chat_type": "direct",
  "participant_ids": [3]
}
```

**Request Body (Group Chat):**
```json
{
  "chat_type": "group",
  "name": "Study Group",
  "description": "Group for studying together",
  "participant_ids": [3, 4, 5]
}
```

**Chat Types:**
- `direct`: One-on-one conversation (exactly 1 participant required)
- `group`: Multi-user conversation (name required, 1+ participants)

**Response:** `201 Created`
```json
{
  "id": 1,
  "name": "Study Group",
  "chat_type": "group"
}
```

**Example:**
```bash
# Create direct chat
curl -X POST http://localhost:80/api/v1/chats/ \
  -H "Content-Type: application/json" \
  -b "jwt={jwt}" \
  -d '{"chat_type": "direct", "participant_ids": [3]}'

# Create group chat
curl -X POST http://localhost:80/api/v1/chats/ \
  -H "Content-Type: application/json" \
  -b "jwt={jwt}" \
  -d '{
    "chat_type": "group",
    "name": "Math Group",
    "description": "Math study group",
    "participant_ids": [3, 4, 5]
  }'
```

---

#### `GET /api/v1/chat_messages/`
Get chat message history.

**Query Parameters:**
- `chat_id` (required): Chat ID to get messages from
- `message` (optional): Filter by message content

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "chat_id": 1,
    "sender_id": 2,
    "message": "Hello everyone!",
    "type_message": "text",
    "file_path": null,
    "file_name": null,
    "file_size": null,
    "is_deleted": false,
    "reply_to_id": null,
    "created_at": "2025-01-15T14:00:00Z",
    "updated_at": "2025-01-15T14:00:00Z",
    "sender_name": "John Doe",
    "sender_photo": "http://localhost:80/uploads/profile_pictures/uuid.jpg"
  }
]
```

**Example:**
```bash
curl -X GET "http://localhost:80/api/v1/chat_messages/?chat_id=1" -b "jwt={jwt}"
```

---

#### `POST /api/v1/chat_messages/`
Send a text message to a chat.

**Request Body:**
```json
{
  "chat_id": 1,
  "message": "Hello everyone!",
  "reply_to_id": null
}
```

**Response:** `201 Created`

**Example:**
```bash
curl -X POST http://localhost:80/api/v1/chat_messages/ \
  -H "Content-Type: application/json" \
  -b "jwt={jwt}" \
  -d '{"chat_id": 1, "message": "Hello!"}'
```

---

#### `POST /api/v1/chat_files/upload/`
Upload a file to a chat.

**Form Data:**
- `chat_id`: Chat ID (required)
- `file`: File to upload (required)
- `reply_to_id`: Message ID to reply to (optional)

**Response:** `201 Created`

**Example:**
```bash
curl -X POST http://localhost:80/api/v1/chat_files/upload/ \
  -b "jwt={jwt}" \
  -F "chat_id=1" \
  -F "file=@document.pdf"
```

---

#### `PUT /api/v1/chat_participants/`
Add participants to a group chat.

**Request Body:**
```json
{
  "chat_id": 1,
  "user_ids": [6, 7]
}
```

**Response:** `200 OK`

**Example:**
```bash
curl -X PUT http://localhost:80/api/v1/chat_participants/ \
  -H "Content-Type: application/json" \
  -b "jwt={jwt}" \
  -d '{"chat_id": 1, "user_ids": [6, 7]}'
```

---

#### `DELETE /api/v1/chat_participants/{user_id}`
Remove a participant from a chat.

**Query Parameters:**
- `chat_id` (required): Chat ID

**Response:** `200 OK`

**Example:**
```bash
curl -X DELETE "http://localhost:80/api/v1/chat_participants/6?chat_id=1" -b "jwt={jwt}"
```

---

#### `PUT /api/v1/chats/{id}/mark_read/`
Mark all messages in a chat as read.

**Response:** `200 OK`

**Example:**
```bash
curl -X PUT http://localhost:80/api/v1/chats/1/mark_read/ -b "jwt={jwt}"
```

---

#### `GET /api/v1/users/available/`
Get list of users available for chat creation.

**Response:** `200 OK`
```json
[
  {
    "id": 3,
    "full_name": "Jane Doe",
    "photo": "http://localhost:80/uploads/profile_pictures/uuid.jpg"
  }
]
```

---

### WebSocket API

Connect to `/api/v1/ws/chat/` for real-time messaging capabilities.

#### Connection

```javascript
const ws = new WebSocket('wss://localhost/api/v1/ws/chat/');
// JWT cookie must be set for authentication
```

#### Client → Server Messages

**Send Message:**
```json
{
  "type": "SendMessage",
  "chat_id": 1,
  "message": "Hello!",
  "reply_to_id": null
}
```

**Typing Indicators:**
```json
{ "type": "TypingStart", "chat_id": 1 }
{ "type": "TypingStop", "chat_id": 1 }
```

**Mark as Read:**
```json
{
  "type": "MarkAsRead",
  "message_id": 123
}
```

**Join/Leave Chat Room:**
```json
{ "type": "JoinChat", "chat_id": 1 }
{ "type": "LeaveChat", "chat_id": 1 }
```

**Heartbeat:**
```json
{ "type": "Ping" }
```

#### Server → Client Messages

**New Message:**
```json
{
  "type": "NewMessage",
  "chat_id": 1,
  "message": {
    "id": 123,
    "message": "Hello!",
    "sender_id": 2,
    "created_at": "2025-01-15T14:00:00Z"
  },
  "sender": {
    "id": 2,
    "full_name": "John Doe",
    "photo": "..."
  }
}
```

**Message Read Receipt:**
```json
{
  "type": "MessageRead",
  "message_id": 123,
  "reader_id": 3,
  "read_at": "2025-01-15T14:05:00Z"
}
```

**Typing Indicators:**
```json
{
  "type": "UserTyping",
  "chat_id": 1,
  "user_id": 3,
  "user_name": "Jane Doe"
}
```
```json
{
  "type": "UserStoppedTyping",
  "chat_id": 1,
  "user_id": 3
}
```

**User Online/Offline Status:**
```json
{ "type": "UserOnline", "user_id": 3 }
{ "type": "UserOffline", "user_id": 3 }
```

**Error:**
```json
{
  "type": "Error",
  "message": "Error description"
}
```

**Heartbeat Response:**
```json
{ "type": "Pong" }
```

#### WebSocket Example (JavaScript)

```javascript
const ws = new WebSocket('wss://yourserver.com/api/v1/ws/chat/');

ws.onopen = () => {
  console.log('Connected');
  // Join a chat room
  ws.send(JSON.stringify({ type: 'JoinChat', chat_id: 1 }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  switch (data.type) {
    case 'NewMessage':
      console.log('New message:', data.message);
      break;
    case 'UserTyping':
      console.log(`${data.user_name} is typing...`);
      break;
    case 'UserOnline':
      console.log(`User ${data.user_id} is online`);
      break;
  }
};

// Send a message
ws.send(JSON.stringify({
  type: 'SendMessage',
  chat_id: 1,
  message: 'Hello!',
  reply_to_id: null
}));

// Start typing indicator
ws.send(JSON.stringify({ type: 'TypingStart', chat_id: 1 }));
```

---

## ⚠️ Error Handling

### Common HTTP Status Codes

- **200 OK**: Request successful
- **201 Created**: Resource created successfully
- **400 Bad Request**: Invalid request data or malformed JSON
- **401 Unauthorized**: Missing or invalid JWT token
- **403 Forbidden**: Insufficient permissions for the requested action
- **404 Not Found**: Resource not found
- **413 Payload Too Large**: File upload exceeds 10MB limit
- **422 Unprocessable Entity**: Valid JSON but invalid data
- **500 Internal Server Error**: Server error

### Error Response Format

```json
{
  "error": "Error description here"
}
```

### Common Error Messages

- `"Missing JWT cookie"`: No authentication token provided
- `"Invalid JWT token"`: Token is expired or malformed
- `"Invalid credentials"`: Wrong username/password
- `"Invalid file extension"`: Unsupported file type
- `"File too large. Limit is 10MB"`: File size exceeds limit
- `"Multiple files not allowed"`: More than one file uploaded
- `"Invalid JSON syntax"`: Malformed JSON in request body

---

## 🔒 Security Features

### Authentication Security
- **ES256 JWT**: Elliptic Curve Digital Signature Algorithm
- **HTTP-Only Cookies**: Prevents XSS attacks
- **1-Hour Token Expiry**: Automatic session timeout
- **Role-Based Access Control**: Fine-grained permissions

### File Upload Security
- **MIME Type Validation**: Files verified by content, not extension
- **File Size Limits**: 10MB maximum per file
- **Filename Sanitization**: Prevents directory traversal
- **UUID File Names**: Prevents filename conflicts

### Input Validation
- **JSON Schema Validation**: Strict request body validation
- **SQL Injection Protection**: Parameterized queries via SQLx
- **XSS Prevention**: Input sanitization
- **CORS Configuration**: Cross-origin request handling

---

## 🕒 Timezone Handling

**Important**: All timestamps in the API are in **UTC**. The system handles timezone conversion as follows:

- **Backend**: All dates and times stored and processed in UTC
- **Database**: All timestamps in UTC
- **API Responses**: All timestamps returned in UTC (ISO 8601 format)
- **Frontend Responsibility**: Convert UTC to local timezone for display
- **Self-Assessments**: Due dates automatically filtered based on UTC time

### Date Format Examples

```json
{
  "created_at": "2025-01-15T14:30:00Z",
  "due_date": "2025-12-01",
  "birth_date": "1990-01-15"
}
```

---

## 📈 Automatic Features

### Self-Assessment Auto-Grading
- **Cron Job**: Runs every 15 minutes
- **Auto-Migration**: Expired self-assessments automatically graded
- **Grade Calculation**: Based on correct/incorrect answers
- **Email Notifications**: Grades sent to students (when email configured)

### File Management
- **Automatic Directories**: Upload folders created automatically
- **UUID Naming**: Prevents filename conflicts
- **Path Resolution**: Dynamic URL generation for file access

---

## 🚀 Development & Production

### Development Mode
```bash
DEBUG=true
```
- Allows user registration without admin authentication
- Extended logging and debugging information
- Test user creation endpoint available

### Production Mode
```bash
DEBUG=false
```
- Admin authentication required for user registration
- Minimal logging for security
- Test endpoints disabled

---

## 📝 API Usage Tips

### Best Practices

1. **Always Include JWT**: All authenticated endpoints require the JWT cookie
2. **Check Token Validity**: Use `/verify_token/` to validate sessions
3. **Handle Errors Gracefully**: Check HTTP status codes and error messages
4. **URL Encode Parameters**: Encode special characters in query parameters
5. **Respect File Limits**: Keep uploads under 10MB
6. **Use Appropriate Content-Type**: JSON for data, multipart for files

### Rate Limiting
- No explicit rate limiting implemented
- Server handles concurrent requests efficiently
- Consider implementing client-side request throttling

### Pagination
- Currently not implemented
- Large datasets returned in full
- Consider filtering for better performance

---

## 🔧 Configuration

### Required Environment Variables

```bash
# Database
DATABASE_URL=mysql://user:password@localhost:3306/goschool2025

# File Storage
BASE_PATH=/var/www/goschool/uploads
BASE_URL=http://localhost:80

# Development
DEBUG=true

# JWT Keys (auto-generated by setup script)
# /shared/ecc_private_key.pem
# /shared/ecc_public_key.pem
```

### File Structure
```
/uploads/
  ├── profile_pictures/   # User profile images
  └── files/             # Assignment submissions and materials
```

---

## 🗄️ Database Schema

### Entity Relationship Overview

```
┌─────────────┐     ┌─────────────┐     ┌─────────────────┐
│   users     │────<│   roles     │     │  personal_data  │
└─────────────┘     └─────────────┘     └─────────────────┘
      │                                          │
      │ 1:N                                     1:1
      │                                          │
      ▼                                          ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────────┐
│  courses    │────<│  subjects   │────<│   assessments   │
└─────────────┘     └─────────────┘     └─────────────────┘
      │                   │                      │
      │                   │                      │
      ▼                   ▼                      ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────────┐
│ timetables  │     │   grades    │     │ selfassessables │
└─────────────┘     └─────────────┘     └─────────────────┘
```

### Core Tables

| Table | Description | Key Columns |
|-------|-------------|-------------|
| `users` | User accounts | `id`, `email`, `password`, `course_id`, `photo`, `last_login` |
| `roles` | User role assignments | `id`, `user_id`, `role` (admin/teacher/student/father/preceptor) |
| `personal_data` | User profiles | `id`, `user_id`, `full_name`, `birth_date`, `address`, `phone_number` |
| `families` | Parent-student relations | `id`, `student_id`, `father_id` |

### Academic Tables

| Table | Description | Key Columns |
|-------|-------------|-------------|
| `courses` | Classes/grade levels | `id`, `year`, `division`, `level`, `shift`, `name`, `preceptor_id` |
| `subjects` | Academic subjects | `id`, `name`, `course_id`, `teacher_id` |
| `timetables` | Class schedules | `id`, `course_id`, `subject_id`, `day`, `start_time`, `end_time` |
| `assessments` | Tasks/exams | `id`, `type`, `subject_id`, `task`, `due_date`, `created_at` |
| `grades` | Student grades | `id`, `student_id`, `subject_id`, `assessment_id`, `grade`, `grade_type`, `description` |
| `homework_submissions` | Student submissions | `id`, `task_id`, `student_id`, `path` |

### Self-Assessment Tables

| Table | Description | Key Columns |
|-------|-------------|-------------|
| `selfassessables` | Auto-graded quizzes | `id`, `assessment_id` |
| `selfassessable_tasks` | Quiz questions | `id`, `selfassessable_id`, `question`, `correct`, `incorrect1-4` |
| `selfassessable_submissions` | Student answers | `id`, `selfassessable_id`, `answers`, `student_id` |
| `selfassessable_pending_grades` | Auto-grading queue | `id`, `selfassessable_id`, `student_id`, `grade` |

### Communication Tables

| Table | Description | Key Columns |
|-------|-------------|-------------|
| `messages` | System announcements | `id`, `sender_id`, `message`, `title`, `created_at` |
| `message_courses` | Message distribution | `message_id`, `course_id` |
| `subject_messages` | Subject-specific content | `id`, `sender_id`, `subject_id`, `title`, `content`, `type` |
| `reads` | Message read receipts | `id`, `message_id`, `reader_id`, `when` |

### Chat Tables

| Table | Description | Key Columns |
|-------|-------------|-------------|
| `chats` | Conversations | `id`, `name`, `photo`, `description`, `chat_type`, `created_by` |
| `chat_participants` | Chat membership | `id`, `user_id`, `chat_id`, `joined_at`, `last_read_at`, `is_admin` |
| `chat_messages` | Chat history | `id`, `chat_id`, `sender_id`, `message`, `type_message`, `file_path`, `reply_to_id` |

### Attendance & Discipline Tables

| Table | Description | Key Columns |
|-------|-------------|-------------|
| `assistance` | Attendance records | `id`, `student_id`, `presence`, `date` |
| `disciplinary_sanctions` | Disciplinary records | `id`, `student_id`, `sanction_type`, `quantity`, `description`, `date` |

### Database Enums

```sql
-- Role types
ENUM('admin', 'teacher', 'student', 'father', 'preceptor')

-- Assessment types
ENUM('exam', 'homework', 'project', 'oral', 'remedial', 'selfassessable')

-- Grade types
ENUM('numerical', 'conceptual', 'percentage')

-- Presence status
ENUM('present', 'absent', 'excused', 'late')

-- Chat types
ENUM('direct', 'group')

-- Message types
ENUM('text', 'file')

-- Subject message types
ENUM('message', 'link', 'file')
```

---

## 🛠️ Development Guide

### Prerequisites

- **Rust**: 1.75+ (stable)
- **MySQL**: 8.0+
- **Python**: 3.8+ (for database setup scripts)
- **OpenSSL**: For JWT key generation

### Local Development Setup

```bash
# 1. Clone and navigate to backend
cd back

# 2. Set up environment variables
export DATABASE_URL="mysql://user:password@localhost:3306/goschool2025"
export BASE_URL="http://localhost:80"
export BASE_PATH="/var/www/goschool"
export DEBUG="true"

# 3. Create database and tables
python3 create_database.py create_all

# 4. Run development server
cargo run

# Or build release binary
cargo build --release
./target/release/back
```

### Environment Variables Reference

| Variable | Required | Description | Example |
|----------|----------|-------------|---------|
| `DATABASE_URL` | Yes | MySQL connection string | `mysql://user:pass@localhost:3306/goschool2025` |
| `BASE_URL` | Yes | Public-facing URL | `https://yourserver.com` |
| `BASE_PATH` | Yes | File storage base path | `/var/www/goschool` |
| `DEBUG` | No | Enable debug features | `true` or `false` |
| `EMAIL_FROM` | No | Email sender address | `noreply@school.com` |
| `EMAIL_USERNAME` | No | SMTP username | `smtp_user` |
| `EMAIL_PASSWORD` | No | SMTP password | `smtp_pass` |

### Code Patterns

**Adding a New Endpoint:**

1. Define struct in `structs.rs`:
```rust
#[derive(Serialize, Deserialize)]
pub struct NewResource {
    pub field: String,
}
```

2. Add trait method in `traits.rs`:
```rust
async fn get_resources(&self, pool: &MySqlPool, filter: ResourceFilter) -> Result<Vec<Resource>, sqlx::Error>;
```

3. Implement in `impls/`:
```rust
impl Get for MySelf {
    async fn get_resources(&self, pool: &MySqlPool, filter: ResourceFilter) -> Result<Vec<Resource>, sqlx::Error> {
        sqlx::query_as!(Resource, "SELECT * FROM resources WHERE ...")
            .fetch_all(pool)
            .await
    }
}
```

4. Create handler in `views/`:
```rust
pub async fn get_resources(
    pool: Data<MySqlPool>,
    req: HttpRequest,
    filter: Query<ResourceFilter>,
) -> HttpResponse {
    let jwt = req.cookie("jwt").unwrap();
    let user = get_myself_from_token(&jwt.value()).unwrap();
    match user.get_resources(&pool, filter.into_inner()).await {
        Ok(resources) => HttpResponse::Ok().json(resources),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}
```

5. Register route in `routes.rs`:
```rust
.route("/resources/", web::get().to(get_resources))
```

### Testing

```bash
# Create test users (requires DEBUG=true)
curl http://localhost:80/api/v1/register_testing_users/

# Health check
curl http://localhost:80/api/v1/health/

# Login and save cookie
curl -c cookies.txt -X POST http://localhost:80/api/v1/login/ \
  -H "Content-Type: application/json" \
  -d '{"email": "admin", "password": "admin", "role": "admin"}'

# Use saved cookie for subsequent requests
curl -b cookies.txt http://localhost:80/api/v1/courses/
```

### Cron Jobs

The backend runs automatic scheduled tasks:

| Task | Schedule | Description |
|------|----------|-------------|
| Self-Assessment Grading | Every 15 minutes | Grades expired self-assessments automatically |

```rust
// Cron expression: 0 0/15 * * * * *
// Runs at minute 0, 15, 30, 45 of every hour
```

### File Upload Paths

| Upload Type | Storage Path | URL Path |
|-------------|--------------|----------|
| Profile Pictures | `./uploads/profile_pictures/` | `/uploads/profile_pictures/{uuid}` |
| General Files | `./uploads/files/` | `/uploads/files/{uuid}` |
| Homework Submissions | `./uploads/submissions/` | `/uploads/submissions/{uuid}` |
| Chat Files | `./uploads/chat_files/` | `/uploads/chat_files/{uuid}` |

### Common Issues

**JWT Token Invalid:**
- Ensure JWT keys exist in `/shared/ecc_private_key.pem` and `/shared/ecc_public_key.pem`
- Run `python3 create_database.py create_all` to regenerate keys

**Database Connection Failed:**
- Verify `DATABASE_URL` format: `mysql://user:password@host:port/database`
- Ensure MySQL is running and accessible

**File Upload Fails:**
- Check file size (max 10MB)
- Verify MIME type is supported
- Ensure upload directories exist and are writable

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).

**Project maintained by**: [Valentino Grande](https://github.com/valentinogrande)

---

## 🤝 Support

For issues, questions, or contributions, please visit the project repository or contact the maintainer.

**Remember**: Replace `{jwt}` in examples with actual JWT token values received from the login endpoint.
