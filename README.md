# goschool

# como usar el ./create_database.py
<code>python3 create_database.py delete_tables</code>
<code>python3 create_database.py create_all</code>

el create_all tambien crea la clave rsa para el jwt

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


# endpoints


## verify token 
<code>curl -v -X GET "http://localhost:8080/api/v1/verify_token/" -b "jwt={json web token}"</code>


## get roles for login usa credenciales:
<code>curl -X POST http://localhost:8000/api/v1/get_roles/ -H "Content-Type: application/json" -d '{"email": "admin", "password": "admin"}'</code>

## login:
<code>curl -v -X POST http://localhost:8080/api/v1/login/ -H "Content-Type: application/json" -d '{"email": "father", "password": "father", "role": "father"}'</code>

## logout:
<code>curl -v -X POST "http://localhost:8080/api/v1/logout/"</code>

## obtener el role usa jwt:
<code>curl -v -X GET "http://localhost:8080/api/v1/get_role/"</code>

## register:
<code>curl -X POST http://localhost:8080/api/v1/register/ -H "Content-Type: application/json" -b "jwt=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." -d '{
    "name": "Juan Pérez",
    "email": "juan@example.com",
    "password": "superseguro123",
    "role": "teacher"
  }'</code>

## subir la foto de perfil: 
<code>curl -v -X POST "http://localhost:8080/api/v1/upload_profile_picture/" -b "jwt={json web token}" -F "image=@{image.path}"</code>

## obtener link de la foto:
<code>curl -v -X GET "http://localhost:8080/api/v1/get_profile_picture/" -b "jwt={json web token}"</code>

## crear una evaluacion:
<code>curl -v -X POST http://localhost:8080/api/v1/create_assessment/ -H "Content-Type: application/json" -H "Cookie: jwt={json web token}" -d '{
    "task": "upload 2+2",
    "subject": 1,
    "type": "homework",
    "due_date": "2026-04-15"
}'</code>


## subir una respuesta a una tarea previamente creada por un profesor:
<code>curl -v http://localhost:8080/api/v1/create_submission/ -H "Cookie: jwt={json web token}" -F "homework=@test.pdf" -F "homework_id=1"</code>

## subir una nota:
<code>curl -v -X POST http://localhost:8080/api/v1/assign_grade/ -H "Content-Type: application/json" -b "jwt={json web token}" -d '{
    "subject": 1,
    "assessment_id": 1, # en caso de no tener una evauliacion de referencia usar "null", ejemplo nota de comportamineto
    "student_id": 2,
    "grade_type": "numerical",
    "description": "prueba de integrales y derivadas",
    "grade": 4.5
  }'</code>

## obtener las evaluaciones de un alumno (en caso de no tener id osea referenciarse a uno mismo poner 0 en el id):
<code>curl -v http://localhost:8080/api/v1/get_student_assessments_by_id/{user_id}/ -G --data-urlencode "subject_id=1" --data-urlencode "task=2+2" --data-urlencode "due=true"</code>
importante en caso de no querer filtar eliminar el parametro de la url

## obtener las notas de un alumno (en caso de no tener id osea referenciarse a uno mismo poner 0 en el id):
<code>curl -v -G "http://localhost:8080/api/v1/get_student_grades_by_id/{user_id}/" --data-urlencode "subject_id=1" --data-urlencode "description=prueba" -b "jwt={json web token}"</code>

## obtener la informacion personal (full_name, birth_date, address, phone_number)
<code>curl -v -X GET "http://localhost:8080/api/v1/get_personal_data/" -b "jwt={json web token}"</code>
