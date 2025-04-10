import mysql.connector
import sys
from datetime import datetime

current_year = datetime.now().year

conn = mysql.connector.connect(
    host='localhost',
    user='root',
    password='mili2009',
    database=f'colegio_stella_maris_{current_year}'
)

cursor = conn.cursor()


with open('database.sql', 'r') as file:
    

    if len(sys.argv) > 1:
        command = sys.argv[1]

        if command == "create_courses":
            
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
        if command == "delete_tables":
            cursor.execute("SET FOREIGN_KEY_CHECKS = 0")

            cursor.execute("SHOW TABLES")
            tables = cursor.fetchall()

            for (table_name,) in tables:
                print(f"Borrando tabla: {table_name}")
                cursor.execute(f"DROP TABLE IF EXISTS `{table_name}`")
            cursor.execute("SET FOREIGN_KEY_CHECKS = 1")

        if command == "create_tables":
            sql_script = file.read()
            commands = sql_script.split(';')

            for command in commands:
                command = command.strip()
                if command.startswith('--'):
                    continue
                if command:
                    try:
                        table = command.split()[5]
                        print(f'creating table: {table}')
                        cursor.execute(command)
                    except mysql.connector.Error as err:
                        print(f'Error: {err}')
        if command == "create_users":
            res=__import__("requests").get("http://localhost:8080/api/v1/register_testing_users/")
            if res.status_code == 201:
                print("users created succesfully")
                

conn.commit()
cursor.close()
conn.close()
