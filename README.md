# goschool

# como usar el ./create_database.py
<code>python3 create_database.py comando</code>
remplazar comando por:
<br>
create_courses
<br>
delete_tables
<br>
create_tables
<br>
create_users

## usuarios
admin: admin
<br>
student: student
<br>
father: father
<br>
teacher: teacher
<br>
preceptor: preceptor

# !! imporatante
el create_courses no es relevante ahora.
para actualizar la db poner los comandos {esto va a eliminar todo y lo vuelve a crear.}
<code>python3 create_database.py delete_tables ;python3 create_database.py create_tables ;python3 create_database.py create_users</code>



## endpoints


### verify token 
<code>curl -v -X GET "http://localhost:8080/api/v1/verify_token/" -b "jwt={json web token}"<code>

### login:
<code>curl -v -X POST "http://localhost:8080/api/v1/login/" -H "Content-Type: application/json" -d '{"email": "teacher", "password": "teacher"}'</code>

### subir la foto de perfil: 
<code>curl -v -X POST "http://localhost:8080/api/v1/upload_profile_picture/" -b "jwt={json web token}" -F "image=@{image.path}"</code>

### obtener link de la foto:
<code>curl -v -X GET "http://localhost:8080/api/v1/get_profile_picture" -b "jwt={json web token}"</code>

### crear una evaluacion:
<code>curl -v -X POST http://localhost:8080/api/v1/create_assessment/ -H "Content-Type: application/json" -H "Cookie: jwt={json web token}" -d '{
    "task": "upload 2+2",
    "subject": 1,
    "type_": "homework",
    "due_date": "2026-04-15"
}'</code>


### subir una respuesta a una tarea previamente creada por un profesor:
<code>curl -v http://localhost:8080/api/v1/create_submission/ -H "Cookie: jwt={json web token}" -F "homework=@test.pdf" -F "homework_id=1"</code>

### subir una nota:
<code>curl -v -X POST http://localhost:8080/api/v1/assign_grade/ -H "Content-Type: application/json" -b "jwt={json web token}" -d '{
    "subject": 1,
    "assessment_id": 1, # en caso de no tener una evauliacion de referencia usar "null", ejemplo nota de comportamineto
    "student_id": 2,
    "grade_type": "numerical",
    "description": "prueba de integrales y derivadas",
    "grade": 4.5
  }'</code>

