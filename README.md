# goschool


### mensaje para fran 😎​😎
actualizalo aca el front desp

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
