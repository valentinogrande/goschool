--CREATE DATABASE IF NOT EXISTS colegio_stella_maris_2025;

--USE colegio_stella_maris_2025;


CREATE TABLE IF NOT EXISTS users (
  id BIGINT AUTO_INCREMENT PRIMARY KEY,
  email VARCHAR(255) NOT NULL UNIQUE,
  password VARCHAR(255) NOT NULL,
  role ENUM('admin', 'teacher', 'student', 'preceptor') DEFAULT 'student',
  last_login TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS personal_data (
  id BIGINT AUTO_INCREMENT PRIMARY KEY,
  user BIGINT NOT NULL,
  full_name VARCHAR(255) NOT NULL,
  birth_date DATE NOT NULL,
  address VARCHAR(255) NOT NULL,
  phone_number VARCHAR(20) NOT NULL,
  FOREIGN KEY (user) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS courses (
  id BIGINT AUTO_INCREMENT PRIMARY KEY,
  year INT NOT NULL,
  division CHAR(1) NOT NULL,
  level ENUM('primary', 'secondary') NOT NULL DEFAULT 'secondary',
  shift ENUM('morning', 'afternoon') NOT NULL DEFAULT 'morning'
);

CREATE TABLE IF NOT EXISTS teachers_courses (
  id BIGINT AUTO_INCREMENT PRIMARY KEY,
  teacher_id BIGINT NOT NULL,
  course_id BIGINT NOT NULL,
  FOREIGN KEY (teacher_id) REFERENCES users(id),
  FOREIGN KEY (course_id) REFERENCES courses(id)
);

CREATE TABLE IF NOT EXISTS students_courses (
  student_id BIGINT NOT NULL,
  course_id BIGINT NOT NULL,
  PRIMARY KEY (student_id, course_id),
  FOREIGN KEY (student_id) REFERENCES users(id),
  FOREIGN KEY (course_id) REFERENCES courses(id)
);

CREATE TABLE IF NOT EXISTS preceptor_courses (
  id BIGINT AUTO_INCREMENT PRIMARY KEY,
  preceptor_id BIGINT NOT NULL,
  course_id BIGINT NOT NULL,
  FOREIGN KEY (preceptor_id) REFERENCES users(id),
  FOREIGN KEY (course_id) REFERENCES courses(id)
);

CREATE TABLE IF NOT EXISTS subjects (
  id BIGINT AUTO_INCREMENT PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  course_id BIGINT NOT NULL,
  teacher_id BIGINT NOT NULL,
  FOREIGN KEY (course_id) REFERENCES courses(id),
  FOREIGN KEY (teacher_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS timetables (
  id BIGINT AUTO_INCREMENT PRIMARY KEY,
  course_id BIGINT NOT NULL,
  subject BIGINT NOT NULL,
  start_time TIME NOT NULL,
  end_time TIME NOT NULL,
  day ENUM('Monday','Tuesday','Wednesday','Thursday','Friday') NOT NULL,
  FOREIGN KEY (course_id) REFERENCES courses(id),
  FOREIGN KEY (subject) REFERENCES subjects(id)
);

CREATE TABLE IF NOT EXISTS grades (
  id BIGINT AUTO_INCREMENT PRIMARY KEY,
  desciption TEXT,
  grade DECIMAL(5,2) NOT NULL,
  student_id BIGINT NOT NULL,
  subject_id BIGINT NOT NULL,
  reference_id BIGINT NOT NULL,
  note_type ENUM('numerical','conceptual','percentage',) DEFAULT 'numerical',
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (student_id) REFERENCES users(id),
  FOREIGN KEY (subject_id) REFERENCES subjects(id),
  FOREIGN KEY (reference_id) REFERENCES assessments(id)
);
 
CREATE TABLE IF NOT EXISTS assessments (
  id BIGINT AUTO_INCREMENT PRIMARY KEY,
  type ENUM('exam','homework','project') DEFAULT 'exam',
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  due_date DATE NOT NULL,
  task TEXT NOT NULL,
  subject_id BIGINT NOT NULL,
  FOREIGN KEY (subject_id) REFERENCES subjects(id)
);
 
CREATE TABLE IF NOT EXISTS homework_submissions (
  id BIGINT AUTO_INCREMENT PRIMARY KEY,
  task_id BIGINT NOT NULL,
  student_id BIGINT NOT NULL,
  path VARCHAR(255) NOT NULL,
  FOREIGN KEY (task_id) REFERENCES assessments(id),
  FOREIGN KEY (student_id) REFERENCES users(id)
);
