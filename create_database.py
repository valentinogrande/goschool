import mysql.connector
import sys
from datetime import datetime
import requests

current_year = datetime.now().year

conn = mysql.connector.connect(
    host='localhost',
    user='root',
    password='mili2009',
    database=f'colegio_stella_maris_{current_year}'
)

cursor = conn.cursor()

def create_courses():
    print()
    print("creating courses")
    print()
    years = int(input("Enter the number of years: "))
    divisions = int(input("Enter the number of divisions per year: "))
    primary = int(input("Enter the number of primary levels: "))

    for i in range(years):
        for j in range(divisions):
            level = "secondary" if i >= primary else "primary"
            shift = "morning" if level == "secondary" or j == 2 else "afternoon"

            cursor.execute(
                "INSERT INTO courses (year, division, level, shift) VALUES (%s, %s, %s, %s)",
                (i + 1, j + 1, level, shift)
            )

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

def create_tables(file):
    print()
    print("creating tables")
    print()
    sql_script = file.read()
    commands = sql_script.split(';')

    for command in commands:
        command = command.strip()
        if command.startswith('--'):
            continue
        if command:
            try:
                if command.startswith("ALTER"):
                    pass
                else:
                    table = command.split()[5]
                    print(f'creating table: {table}')
                cursor.execute(command)
            except mysql.connector.Error as err:
                print(f'Error: {err}')

def create_users():
    print()
    print("creating users")
    print()
    res=requests.get("http://localhost:8080/api/v1/register_testing_users/")
    if res.status_code == 201:
        print("users created succesfully")
        cursor.execute("INSERT INTO families (student_id, father_id) VALUES (%s,%s)",(2,4))
        cursor.execute("INSERT INTO personal_data (user_id, full_name, birth_date, address, phone_number) VALUES (%s, %s, %s, %s, %s)", (1,"admin","2000-01-01","mi casa","123456789"))
        cursor.execute("INSERT INTO personal_data (user_id, full_name, birth_date, address, phone_number) VALUES (%s, %s, %s, %s, %s)", (2,"student","2000-01-01","mi casa","123456789"))
        cursor.execute("INSERT INTO personal_data (user_id, full_name, birth_date, address, phone_number) VALUES (%s, %s, %s, %s, %s)", (3,"preceptor","2000-01-01","mi casa","123456789"))
        cursor.execute("INSERT INTO personal_data (user_id, full_name, birth_date, address, phone_number) VALUES (%s, %s, %s, %s, %s)", (4,"father","2000-01-01","mi casa","123456789"))
        cursor.execute("INSERT INTO personal_data (user_id, full_name, birth_date, address, phone_number) VALUES (%s, %s, %s, %s, %s)", (5,"teacher","2000-01-01","mi casa","123456789"))
        cursor.execute("INSERT INTO subjects (name, course_id, teacher_id) VALUES ('matematica',34,5)")
        cursor.execute("UPDATE users SET course_id=34 WHERE id=2")


def create_preceptors():
    print()
    print("making preceptors")
    print()
    cursor.execute("UPDATE courses SET preceptor_id=3 WHERE id=34")
    cursor.execute("UPDATE courses SET preceptor_id=3 WHERE id=35")
    cursor.execute("UPDATE courses SET preceptor_id=3 WHERE id=36")
    print("preceptors courses were addded succesfully")


with open('database.sql', 'r') as file:
    

    if len(sys.argv) > 1:
        command = sys.argv[1]

        if command == "create_courses":
            create_courses()
        if command == "delete_tables":
            delete_tables()
        if command == "create_tables":
            create_tables(file)
        if command == "create_users":
            create_users()
        if command == "create_preceptors":
            create_preceptors()
        if command == "create_all":
            create_tables(file)
            create_courses()
            create_users()
            create_preceptors()
            print("\033[92mAll tables created succesfully\033[0m")
    
conn.commit()
cursor.close()
conn.close()
