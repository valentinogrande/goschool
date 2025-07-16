import random
from faker import Faker
import bcrypt
from datetime import datetime, timedelta, date
import mysql.connector
from mysql.connector import Error
import time

# Configuración
fake = Faker('es_AR')
current_year = datetime.now().year

# Configuración de la base de datos
DB_CONFIG = {
    'host': 'localhost',
    'user': 'root',
    'password': 'mili2009',
    'database': f'colegio_stella_maris_{current_year}'
}

def create_db_connection():
    """Crea una conexión a la base de datos"""
    try:
        connection = mysql.connector.connect(**DB_CONFIG)
        return connection
    except Error as e:
        print(f"Error conectando a MySQL: {e}")
        return None

def hash_password(password: str) -> str:
    """Hashea la contraseña compatible con Rust (bcrypt)"""
    return "$2b$12$RRSdGpNPikVY/TjNRCcEJ.gG8aqZR4cqY.afGGBblmQp.tB0tpoh6" # -> admin

def register_user(cursor, email: str, password: str, role: str):
    """Registra un usuario directamente en la base de datos"""
    hashed_pass = hash_password(password)
    
    try:
        # Insertar usuario
        cursor.execute(
            "INSERT INTO users (email, password) VALUES (%s, %s)",
            (email, hashed_pass)
        )
        user_id = cursor.lastrowid
        
        # Insertar rol
        cursor.execute(
            "INSERT INTO roles (user_id, role) VALUES (%s, %s)",
            (user_id, role)
        )
        
        return user_id
    except Error as e:
        print(f"Error registrando usuario {email}: {e}")
        return None

def generate_fake_data():
    """Genera datos falsos insertando directamente en la BD"""
    connection = create_db_connection()
    if not connection:
        return

    cursor = connection.cursor()

    try:
        # Listas de datos
        subjects_primary = [
            'Matemática', 'Lengua', 'Ciencias Naturales', 'Ciencias Sociales',
            'Educación Física', 'Educación Artística', 'Educación Musical',
            'Educación Tecnológica', 'Inglés'
        ]
        
        subjects_secondary = [
            'Matemática', 'Lengua y Literatura', 'Historia', 'Geografía',
            'Biología', 'Física', 'Química', 'Educación Física',
            'Inglés', 'Francés', 'Educación Artística', 'Educación Musical',
            'Educación Tecnológica', 'Formación Ética y Ciudadana',
            'Psicología', 'Filosofía', 'Economía'
        ]
        
        assessment_tasks = [
            'Examen parcial de la unidad 1',
            'Trabajo práctico sobre el tema estudiado',
            'Proyecto de investigación grupal',
            'Evaluación oral individual',
            'Informe de laboratorio',
            'Análisis de texto literario',
            'Resolución de problemas matemáticos',
            'Presentación oral del tema',
            'Cuestionario de repaso',
            'Ensayo argumentativo'
        ]
        
        sanction_descriptions = [
            'Llegada tarde reiterada',
            'Falta de respeto a compañeros',
            'No cumplimiento de tareas',
            'Uso inadecuado del uniforme',
            'Comportamiento disruptivo en clase',
            'Falta de materiales escolares',
            'Incumplimiento de normas'
        ]
        
        message_titles = [
            'Reunión de padres',
            'Actividad deportiva',
            'Evento cultural',
            'Comunicado importante',
            'Recordatorio de fechas',
            'Cambio de horarios',
            'Información académica'
        ]

        # 1. Crear admin si no existe
        admin_email = "admin@stellamaris.edu.ar"
        cursor.execute("SELECT id FROM users WHERE email = %s", (admin_email,))
        if not cursor.fetchone():
            admin_id = register_user(cursor, admin_email, "admin123", "admin")
            print(f"Admin creado con ID: {admin_id}")
            age = random.randint(18, 30)
            birth_date = fake.date_of_birth(minimum_age=age, maximum_age=age)
            
            cursor.execute(
                """INSERT INTO personal_data 
                (user_id, full_name, birth_date, address, phone_number)
                VALUES (%s, %s, %s, %s, %s)""",
                (admin_id, fake.name(), birth_date, fake.address(), fake.phone_number())
            )
            connection.commit()

        # 2. Crear profesores (15)
        teachers = []
        for i in range(15):
            email = f"{fake.first_name().lower()}.{fake.last_name().lower()}.prof{i}@stellamaris.edu.ar"
            teacher_id = register_user(cursor, email, "Profesor123", "teacher")
            if teacher_id:
                teachers.append({'id': teacher_id, 'email': email})
                print(f"Profesor creado: {email}")
                age = random.randint(18, 30)
                birth_date = fake.date_of_birth(minimum_age=age, maximum_age=age)
                
                cursor.execute(
                    """INSERT INTO personal_data 
                    (user_id, full_name, birth_date, address, phone_number)
                    VALUES (%s, %s, %s, %s, %s)""",
                    (teacher_id, fake.name(), birth_date, fake.address(), fake.phone_number())
                )
        
        connection.commit()

        # 3. Crear preceptores (1 por curso)
        preceptors = []
        for i in range(7):  # Para 7 años
            email = f"{fake.first_name().lower()}.{fake.last_name().lower()}.prec{i}@stellamaris.edu.ar"
            preceptor_id = register_user(cursor, email, "Preceptor123", "preceptor")
            if preceptor_id:
                preceptors.append({'id': preceptor_id, 'email': email})
                print(f"Preceptor creado: {email}")

                age = random.randint(18, 30)
                birth_date = fake.date_of_birth(minimum_age=age, maximum_age=age)
                
                cursor.execute(
                    """INSERT INTO personal_data 
                    (user_id, full_name, birth_date, address, phone_number)
                    VALUES (%s, %s, %s, %s, %s)""",
                    (preceptor_id, fake.name(), birth_date, fake.address(), fake.phone_number())
                )
        
        connection.commit()

        # 4. Crear cursos (7 años × 3 divisiones)
        courses = []
        for year in range(1, 8):  # 1° a 7° año
            for division in ['A', 'B', 'C']:
                level = 'primary' if year <= 6 else 'secondary'
                shift = random.choice(['morning', 'afternoon'])
                
                # Asignar preceptor
                preceptor = preceptors[year-1] if year-1 < len(preceptors) else None
                
                cursor.execute(
                    "INSERT INTO courses (year, division, level, shift, preceptor_id) VALUES (%s, %s, %s, %s, %s)",
                    (year, division, level, shift, preceptor['id'] if preceptor else None)
                )
                course_id = cursor.lastrowid
                courses.append({
                    'id': course_id,
                    'year': year,
                    'division': division,
                    'level': level,
                    'shift': shift,
                    'preceptor_id': preceptor['id'] if preceptor else None
                })
                print(f"Curso creado: {year}°{division} ({shift}) con preceptor: {preceptor['email'] if preceptor else 'Ninguno'}")
        
        connection.commit()

        # 5. Crear estudiantes (25-35 por curso) y padres
        students = []
        for course in courses:
            for i in range(random.randint(25, 35)):
                # Estudiante
                student_email = f"est.{fake.last_name().lower()}.{course['year']}{course['division']}.{i}@stellamaris.edu.ar"
                student_id = register_user(cursor, student_email, "Estudiante123", "student")
                
                if not student_id:
                    continue
                
                # Datos personales del estudiante
                age = 6 + course['year']  # 6 años para 1er grado
                birth_date = fake.date_of_birth(minimum_age=age, maximum_age=age)
                
                cursor.execute(
                    """INSERT INTO personal_data 
                    (user_id, full_name, birth_date, address, phone_number)
                    VALUES (%s, %s, %s, %s, %s)""",
                    (student_id, fake.name(), birth_date, fake.address(), fake.phone_number())
                )
                
                # Asignar curso al estudiante
                cursor.execute(
                    "UPDATE users SET course_id = %s WHERE id = %s",
                    (course['id'], student_id)
                )
                
                # Padre/Madre
                parent_email = f"padre.{fake.last_name().lower()}.{course['year']}{course['division']}.{i}@gmail.com"
                parent_id = register_user(cursor, parent_email, "Padre123", "father")
                
                if parent_id:
                    # Datos personales del padre
                    cursor.execute(
                        """INSERT INTO personal_data 
                        (user_id, full_name, birth_date, address, phone_number)
                        VALUES (%s, %s, %s, %s, %s)""",
                        (parent_id, fake.name(), 
                         fake.date_of_birth(minimum_age=30, maximum_age=60),
                         fake.address(), fake.phone_number())
                    )
                    
                    # Relación familiar
                    cursor.execute(
                        "INSERT INTO families (student_id, father_id) VALUES (%s, %s)",
                        (student_id, parent_id)
                    )
                
                students.append({'id': student_id, 'course_id': course['id']})
                print(f"Estudiante {student_email} creado para curso {course['year']}°{course['division']}")
            
            connection.commit()

        # 6. Crear materias y asignar profesores
        subjects = []
        for course in courses:
            subject_list = subjects_primary if course['level'] == 'primary' else subjects_secondary
            
            for subject_name in subject_list:
                teacher = random.choice(teachers) if teachers else None
                
                cursor.execute(
                    "INSERT INTO subjects (name, course_id, teacher_id) VALUES (%s, %s, %s)",
                    (subject_name, course['id'], teacher['id'] if teacher else None)
                )
                subject_id = cursor.lastrowid
                subjects.append({
                    'id': subject_id,
                    'name': subject_name,
                    'course_id': course['id'],
                    'teacher_id': teacher['id'] if teacher else None
                })
                print(f"Materia {subject_name} creada para curso {course['year']}°{course['division']} con profesor {teacher['email'] if teacher else 'Ninguno'}")
        
        connection.commit()

        # 7. Crear horarios
        days = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday']
        time_slots_morning = [
            ('08:00:00', '08:40:00'), ('08:40:00', '09:20:00'),
            ('09:20:00', '10:00:00'), ('10:20:00', '11:00:00'),
            ('11:00:00', '11:40:00'), ('11:40:00', '12:20:00')
        ]
        time_slots_afternoon = [
            ('13:30:00', '14:10:00'), ('14:10:00', '14:50:00'),
            ('14:50:00', '15:30:00'), ('15:50:00', '16:30:00'),
            ('16:30:00', '17:10:00')
        ]
        
        for course in courses:
            course_subjects = [s for s in subjects if s['course_id'] == course['id']]
            time_slots = time_slots_morning if course['shift'] == 'morning' else time_slots_afternoon
            
            for day in days:
                daily_subjects = random.sample(course_subjects, min(len(time_slots), len(course_subjects)))
                
                for i, subject in enumerate(daily_subjects):
                    start_time, end_time = time_slots[i]
                    cursor.execute(
                        """INSERT INTO timetables 
                        (course_id, subject_id, start_time, end_time, day)
                        VALUES (%s, %s, %s, %s, %s)""",
                        (course['id'], subject['id'], start_time, end_time, day)
                    )
        
        connection.commit()

        # 8. Crear evaluaciones
        assessments = []
        for subject in subjects:
            for i in range(random.randint(3, 8)):
                assessment_type = random.choice(['exam', 'homework', 'project', 'oral'])
                due_date = fake.date_between(start_date='-60d', end_date='+30d')
                task = random.choice(assessment_tasks)
                
                cursor.execute(
                    """INSERT INTO assessments 
                    (type, due_date, task, subject_id)
                    VALUES (%s, %s, %s, %s)""",
                    (assessment_type, due_date, task, subject['id'])
                )
                assessment_id = cursor.lastrowid
                assessments.append({
                    'id': assessment_id,
                    'type': assessment_type,
                    'subject_id': subject['id']
                })
        
        connection.commit()

        # 9. Crear calificaciones
        for assessment in assessments:
            subject = next(s for s in subjects if s['id'] == assessment['subject_id'])
            cursor.execute("SELECT id FROM users WHERE course_id = %s", (subject['course_id'],))
            course_students = [row[0] for row in cursor.fetchall()]
            
            for student_id in random.sample(course_students, int(len(course_students)*0.8)):  # 80% con calificación
                grade = round(random.uniform(1, 10), 2)
                cursor.execute(
                    """INSERT INTO grades 
                    (description, grade, student_id, subject_id, assessment_id, grade_type)
                    VALUES (%s, %s, %s, %s, %s, %s)""",
                    (f"Calificación de {assessment['type']}", grade, 
                     student_id, subject['id'], assessment['id'], 'numerical')
                )
        
        connection.commit()

        # # 10. Crear asistencias
        # start_date = date(current_year, 3, 1)  # Inicio del año escolar
        # end_date = date(current_year, 7, 16)   # Fecha actual
        #
        # cursor.execute("SELECT id FROM users WHERE id IN (SELECT user_id FROM roles WHERE role = 'student')")
        #student_ids = [row[0] for row in cursor.fetchall()]
        #
        # for student_id in student_ids:
        #     current_date = start_date
        #     while current_date <= end_date:
        #         if current_date.weekday() < 5:  # Solo días de semana
        #             presence = random.choices(
        #                 ['present', 'absent', 'excused'],
        #                 weights=[85, 10, 5]
        #             )[0]
        #
        #             cursor.execute(
        #                 """INSERT INTO assistance 
        #                 (student_id, presence, date)
        #                 VALUES (%s, %s, %s)""",
        #                 (student_id, presence, current_date)
        #
        #         current_date += timedelta(days=1)
        #
        # connection.commit()

        # 11. Crear sanciones disciplinarias
        # for student_id in student_ids:
        #     if random.random() < 0.3:  # 30% de probabilidad
        #         for _ in range(random.randint(1, 3)):
        #             cursor.execute(
        #                 """INSERT INTO disciplinary_sanctions 
        #                 (student_id, sanction_type, quantity, description, date)
        #                 VALUES (%s, %s, %s, %s, %s)""",
        #                 (student_id, 
        #                  random.choice(['admonition', 'warning', 'free']),
        #                  random.randint(1, 3),
        #                  random.choice(sanction_descriptions),
        #                  fake.date_between(start_date='-120d', end_date='now'))
        #
        # connection.commit()
        
        # 12. Crear mensajes generales
        cursor.execute("""
            SELECT u.id FROM users u
            JOIN roles r ON u.id = r.user_id
            WHERE r.role IN ('teacher', 'preceptor')
        """)
        staff_ids = [row[0] for row in cursor.fetchall()]
        
        for i in range(20):
            cursor.execute(
                """INSERT INTO messages 
                (sender_id, message, title)
                VALUES (%s, %s, %s)""",
                (random.choice(staff_ids + [admin_id]), 
                 fake.text(max_nb_chars=200),
                 random.choice(message_titles))
            )
            message_id = cursor.lastrowid
            
            # Asignar a 1-3 cursos
            for course in random.sample(courses, random.randint(1, 3)):
                cursor.execute(
                    "INSERT INTO message_courses (message_id, course_id) VALUES (%s, %s)",
                    (message_id, course['id']))
        
        connection.commit()

        # 13. Crear mensajes de materias
        for subject in subjects:
            for i in range(random.randint(2, 5)):
                cursor.execute(
                    """INSERT INTO subject_messages 
                    (sender_id, subject_id, title, content, type)
                    VALUES (%s, %s, %s, %s, %s)""",
                    (subject['teacher_id'], subject['id'],
                    f"Información sobre {subject['name']}",
                    fake.text(max_nb_chars=150),
                    random.choice(['message', 'link', 'file'])))
        
        connection.commit()

        # Resumen final
        print("\nResumen de datos generados:")
        print(f"- Cursos: {len(courses)} (1° a 7° año, 3 divisiones)")
        print(f"- Profesores: {len(teachers)}")
        print(f"- Preceptores: {len(preceptors)}")
        print(f"- Estudiantes: ~{len(courses)*30} (aprox)")
        print(f"- Materias: {len(subjects)}")
        print(f"- Evaluaciones: {len(assessments)}")
        print(f"- Mensajes: 20 generales + {len(subjects)*3} de materias")

    finally:
        cursor.close()
        connection.close()

if __name__ == "__main__":
    generate_fake_data()
