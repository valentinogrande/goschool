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



# endpoints
login:
<code>curl -v -X POST "http://localhost:8080/api/v1/login/" -H "Content-Type: application/json" -d '{"email": "teacher", "password": "teacher"}'</code>

subir la foto de perfil: 
<code>curl -v -X POST "http://localhost:8080/api/v1/upload_profile_picture/" -b "jwt={json web token}" -F "image=@{image.path}"</code>

obtener link de la foto:
<code>curl -v -X GET "http://localhost:8080/api/v1/get_profile_picture" -b "jwt={jwt}"</code>

crear una evaluacion:
<code>curl -v -X POST http://localhost:8080/api/v1/create_assessment/ \                               -H "Content-Type: application/json" \
  -H "Cookie: jwt={json web token}" \
  -d '{
    "task": "upload 2+2",
    "subject": 1,
    "type_": "homework",
    "due_date": "2026-04-15"
}'</code>


subir una tarea:
<code>curl -v http://localhost:8080/api/v1/create_submission/ \                                 
  -H "Cookie: jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWJqZWN0IjoyLCJleHAiOjE3NDQ0MDY3NDV9.4AdkkX-4oxopHU-Vm7j5fTDS_zp9hyGDfbFUeN1TX2g" \
  -F "homework=@test.pdf" \
  -F "homework_id=1"</code>


