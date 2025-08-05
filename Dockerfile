# Etapa 1: compilación
FROM rust:1.88 as builder

WORKDIR /app

COPY . .

# Instalar dependencias necesarias (ejemplo: OpenSSL para SQLx)
RUN apt update && apt install -y pkg-config libssl-dev

# Compilar en modo release
RUN cargo build --release

# Etapa 2: imagen liviana para producción
FROM debian:bookworm-slim

WORKDIR /app

# Instalar OpenSSL
RUN apt-get update && apt-get install -y libssl3

# Instalar curl
RUN apt-get update && apt-get install -y curl

# Copiar el binario compilado y los email templates
COPY --from=builder /app/target/release/goschool ./app
COPY --from=builder /app/email_templates /app/email_templates

# Crear usuario no root y usarlo
RUN useradd -m appuser

RUN mkdir -p /app/uploads/profile_pictures /app/uploads/files

# Dar propiedad total a appuser sobre todo el directorio /app
RUN chown -R appuser:appuser /app
RUN chmod -R u+rwX /app

USER appuser



ENV RUST_LOG=info

CMD ["./app"]
