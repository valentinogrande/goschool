# goschool

# como usar el ./create_database.py
<code>python3 create_database.py comando</code>
remplazar comando por:
    create_courses
    delete_tables
    create_tables
    create_users

# !! imporatante
el create_courses no es relevante ahora.
para actualizar la db poner los comandos {esto va a eliminar todo y lo vuelve a crear.}
<code>python3 create_database.py delete_tables ;python3 create_database.py create_tables ;python3 create_database.py create_users</code>


subir la foto de perfil: 
<code>curl -v -X POST "http://localhost:8080/api/v1/upload_profile_picture/" -b "jwt={json web token}" -F "image=@{image.path}"</code>

obtener link de la foto:
<code>curl -v -X GET "http://localhost:8080/api/v1/get_profile_picture" -b "jwt={jwt}"</code>


## importtante!!!!
como ver la documentacion de la api??

entra a: <u>http://127.0.0.1:8080/swagger-ui/<u/>

## esto no deberia estar en prod(aviso)

# como ejecutar

te pones en el directorio goscgool y ejectutas:

<code>cargo run</code>

obiamente tenes q tener instalado cargo

# como subir un archivo
<code>curl -v -X POST http://localhost:8080/api/v1/create_submission/ -b "jwt={json web token}" -F "task_id={task id}" -F "task=@{file name}"</code>

### la api esta en <u>http://127.0.0.1/api/v1/</u>
