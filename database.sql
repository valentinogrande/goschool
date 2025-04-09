CREATE DATABASE IF NOT EXISTS colegio_stella_maris_2025;

USE colegio_stella_maris_2025;

CREATE TABLE IF NOT EXISTS users (
  id INT AUTO_INCREMENT PRIMARY KEY,
  email VARCHAR(255) NOT NULL UNIQUE,
  password VARCHAR(255) NOT NULL,
  role ENUM('admin', 'teacher', 'student', 'preceptor') DEFAULT 'student'
);

CREATE TABLE IF NOT EXISTS personal_data (
  id INT AUTO_INCREMENT PRIMARY KEY,
  user INT NOT NULL,
  full_name VARCHAR(255) NOT NULL,
  birth_date DATE NOT NULL,
  address VARCHAR(255) NOT NULL,
  phone_number VARCHAR(20) NOT NULL,
  FOREIGN KEY (user) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS courses (
  id INT AUTO_INCREMENT PRIMARY KEY,
  year INT NOT NULL,         
  division CHAR(1) NOT NULL, 
  level ENUM('primary', 'secondary') NOT NULL DEFAULT 'secondary',
  shift ENUM('morning', 'afternoon') NOT NULL DEFAULT 'morning',
);

CREATE TABLE IF NOT EXISTS teachers_courses (
  id INT AUTO_INCREMENT PRIMARY KEY,
  teacher_id INT NOT NULL,
  course_id VARCHAR(15) NOT NULL,
  FOREIGN KEY (teacher_id) REFERENCES users(id),
  FOREIGN KEY (course_id) REFERENCES courses(id)
);

CREATE TABLE IF NOT EXISTS students_courses (
  student_id INT NOT NULL,
  course_id VARCHAR(15) NOT NULL,
  PRIMARY KEY (student_id, course_id),
  FOREIGN KEY (student_id) REFERENCES users(id),
  FOREIGN KEY (course_id) REFERENCES courses(id)
);

CREATE TABLE IF NOT EXISTS preceptor_courses (
  id INT AUTO_INCREMENT PRIMARY KEY,
  preceptor_id INT NOT NULL,
  course_id VARCHAR(15) NOT NULL,
  FOREIGN KEY (preceptor_id) REFERENCES users(id),
  FOREIGN KEY (course_id) REFERENCES courses(id)
);

CREATE TABLE IF NOT EXISTS subjects (
  id INT AUTO_INCREMENT PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  course_id VARCHAR(15) NOT NULL,
  teacher_id INT NOT NULL,
  FOREIGN KEY (course_id) REFERENCES courses(id),
  FOREIGN KEY (teacher_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS timetables (
  id INT AUTO_INCREMENT PRIMARY KEY,
  course_id VARCHAR(15) NOT NULL,
  subject INT NOT NULL,
  start_time TIME NOT NULL,
  end_time TIME NOT NULL,
  day ENUM('Monday','Tuesday','Wednesday','Thursday','Friday') NOT NULL,
  FOREIGN KEY (course_id) REFERENCES courses(id),
  FOREIGN KEY (subject) REFERENCES subjects(id)
);

CREATE TABLE IF NOT EXISTS tasks (
  id INT AUTO_INCREMENT PRIMARY KEY,
  task TEXT NOT NULL,
  teacher_id INT NOT NULL,
  course_id VARCHAR(15) NOT NULL,
  FOREIGN KEY (teacher_id) REFERENCES users(id),
  FOREIGN KEY (course_id) REFERENCES courses(id)
);

CREATE TABLE IF NOT EXISTS tasks_submissions (
  id INT AUTO_INCREMENT PRIMARY KEY,
  task_id INT NOT NULL,
  student_id INT NOT NULL,
  path VARCHAR(255) NOT NULL,
  FOREIGN KEY (task_id) REFERENCES tasks(id),
  FOREIGN KEY (student_id) REFERENCES users(id)
);
