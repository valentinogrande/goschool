import mysql.connector
import sys
from datetime import datetime
import requests
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import ec
import os

#from fake_data import generate_fake_data

def generate_key_pair():
    try: 
        os.remove("/shared/ecc_private_key.pem")
        os.remove("/shared/ecc_public_key.pem")
    except:
        pass
    private_key = ec.generate_private_key(ec.SECP256R1())  # P-256

    with open("/shared/ecc_private_key.pem", "wb") as f:
        f.write(private_key.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.PKCS8,
            encryption_algorithm=serialization.NoEncryption()
        ))

    public_key = private_key.public_key()
    with open("/shared/ecc_public_key.pem", "wb") as f:
        f.write(public_key.public_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PublicFormat.SubjectPublicKeyInfo
        ))
        
    print("\033[92mrsa keys were created\033[0m")


current_year = datetime.now().year

conn = mysql.connector.connect(
    host='mysql',
    user='adminweb',
    password='mili2009',
    database=f'colegio_stella_maris_{current_year}'
)

cursor = conn.cursor()

def get_ordinal_name(year: int) -> str:
    """Devuelve el nombre ordinal (Primero, Segundo, etc.) según el número de año"""
    nombres = [
        "Primer grado", "Segundo grado", "Tercero grado", "Cuarto grado", "Quinto grado", "Sexto grado", "Séptimo grado", #primaria
        "Primer año", "Segundo año", "Tercero año", "Cuarto año", "Quinto año" # secundaria
    ]
    return nombres[year - 1] if 1 <= year <= len(nombres) else f"Año {year}"

def get_division_name(level: str, division: int) -> str:
    """Devuelve el nombre de la división según el nivel y el número de división"""
    if level == "primary":
        divisiones = ["Mar", "Gaviota", "Estrella"]
    else:  # secondary
        divisiones = ["Economía", "Naturales", "Humanidades"]
    return divisiones[division - 1]

def create_courses():
    print("\nCreando cursos...\n")

    years = 12
    divisions = 3
    primary_limit = 7

    for i in range(years):  # 0 a 11
        year_number = i + 1
        level = "primary" if year_number <= primary_limit else "secondary"

        for j in range(divisions):  # 0 a 2
            division_number = j + 1
            shift = "afternoon" if level == "primary" and division_number != 3 else "morning"
            ordinal = get_ordinal_name(year_number)
            division_name = get_division_name(level, division_number)

            name = f"{ordinal} {division_name}"

            cursor.execute(
                "INSERT INTO courses (year, division, level, shift, name) VALUES (%s, %s, %s, %s, %s)",
                (year_number, division_number, level, shift, name)
            )

    conn.commit()
    print("\nTodos los cursos fueron creados.")

def delete_tables():
    print()
    print("deleting tables")
    print()
    cursor.execute("SET FOREIGN_KEY_CHECKS = 0")

    cursor.execute("SHOW TABLES")
    tables = cursor.fetchall()

    for (table_name,) in tables:
        print(f"Borrando tabla: {table_name}")
        cursor.execute(f"DROP TABLE IF EXISTS `{table_name}`")
    cursor.execute("SET FOREIGN_KEY_CHECKS = 1")

# def create_tables(file):
#     print()
#     print("creating tables...")
#     print()
#     sql_script = file.read()
#     commands = sql_script.split(';')
#
#     for command in commands:
#         command = command.strip()
#         if command.startswith('--'):
#             continue
#         if command:
#             try:
#                 if command.startswith("ALTER"):
#                     pass
#                 else:
#                     table = command.split()[5]
#                     print(f'creating table: {table}')
#                 cursor.execute(command)
#             except mysql.connector.Error as err:
#                 print(f'Error: {err}')

def create_users():
    print()
    print("creating users...")
    print()
    res=requests.get("http://backend/api/v1/register_testing_users/")
    if res.status_code == 201:
        print("users created succesfully")
        cursor.execute("INSERT INTO families (student_id, father_id) VALUES (%s,%s)",(2,4))
        cursor.execute("INSERT INTO personal_data (user_id, full_name, birth_date, address, phone_number) VALUES (%s, %s, %s, %s, %s)", (1,"admin","2000-01-01","mi casa","123456789"))
        cursor.execute("INSERT INTO personal_data (user_id, full_name, birth_date, address, phone_number) VALUES (%s, %s, %s, %s, %s)", (2,"student","2000-01-01","mi casa","123456789"))
        cursor.execute("INSERT INTO personal_data (user_id, full_name, birth_date, address, phone_number) VALUES (%s, %s, %s, %s, %s)", (3,"preceptor","2000-01-01","mi casa","123456789"))
        cursor.execute("INSERT INTO personal_data (user_id, full_name, birth_date, address, phone_number) VALUES (%s, %s, %s, %s, %s)", (4,"father","2000-01-01","mi casa","123456789"))
        cursor.execute("INSERT INTO personal_data (user_id, full_name, birth_date, address, phone_number) VALUES (%s, %s, %s, %s, %s)", (5,"teacher","2000-01-01","mi casa","123456789"))
        cursor.execute("INSERT INTO subjects (name, course_id, teacher_id) VALUES ('matematica',34,5)")
        cursor.execute("INSERT INTO subjects (name, course_id, teacher_id) VALUES ('lengua',34,5)")
        cursor.execute("INSERT INTO subjects (name, course_id, teacher_id) VALUES ('historia',34,5)")
        cursor.execute("INSERT INTO subjects (name, course_id, teacher_id) VALUES ('matematica',35,5)")
        cursor.execute("UPDATE users SET course_id=34 WHERE id=2")
        cursor.execute("UPDATE users SET course_id=34 WHERE id=6")


def create_preceptors():
    print()
    print("making preceptors...")
    print()
    cursor.execute("UPDATE courses SET preceptor_id=3 WHERE id=34")
    cursor.execute("UPDATE courses SET preceptor_id=3 WHERE id=35")
    cursor.execute("UPDATE courses SET preceptor_id=3 WHERE id=36")
    print("preceptors courses were addded succesfully")

def create_timetables():
    print()
    print("making timetables...")
    print()
    cursor.execute("INSERT INTO timetables (course_id, subject_id, start_time, end_time, day) VALUES (%s, %s, %s, %s, %s)", (34,1,"07:00:00","13:00:00","Monday"))
    cursor.execute("INSERT INTO timetables (course_id, subject_id, start_time, end_time, day) VALUES (%s, %s, %s, %s, %s)", (34,3,"07:00:00","13:00:00","Tuesday"))
    cursor.execute("INSERT INTO timetables (course_id, subject_id, start_time, end_time, day) VALUES (%s, %s, %s, %s, %s)", (34,1,"07:00:00","13:00:00","Wednesday"))
    cursor.execute("INSERT INTO timetables (course_id, subject_id, start_time, end_time, day) VALUES (%s, %s, %s, %s, %s)", (34,2,"07:00:00","13:00:00","Thursday"))
    cursor.execute("INSERT INTO timetables (course_id, subject_id, start_time, end_time, day) VALUES (%s, %s, %s, %s, %s)", (34,1,"07:00:00","8:00:00","Friday"))
    cursor.execute("INSERT INTO timetables (course_id, subject_id, start_time, end_time, day) VALUES (%s, %s, %s, %s, %s)", (34,2,"08:15:00","9:30:00","Friday"))


def check_already_created():
    print()
    print("\033[91mChecking if the data had been already inserted...\033[0m")
    cursor.execute("SELECT 1 FROM users LIMIT 1")
    data = cursor.fetchone()
    
    if data is None:
        return False
    else:
        return True

#with open('database.sql', 'r') as file:


if len(sys.argv) > 1:
    command = sys.argv[1]

    if command == "create_courses":
        create_courses()
    if command == "delete_tables":
        delete_tables()
    #if command == "create_tables":
        #create_tables(file)
    if command == "create_users":
        create_users()
    if command == "create_preceptors":
        create_preceptors()
    if command == "generate_rsa":
        generate_key_pair()
    if command == "create_timetables":
        create_timetables()
    #if command == "insert_fake_data":
        #generate_fake_data()
    
    if command == "create_all":
        generate_key_pair()
        if not check_already_created(): 
            #create_tables(file)
            create_courses()
            create_users()
            create_preceptors()
            create_timetables()
            print("\033[92mAll tables created succesfully\033[0m")
        else:
            print("\033[91mData had been already inserted, so no changes.\033[0m")

conn.commit()
cursor.close()
conn.close()
