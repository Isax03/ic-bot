FROM rust:bullseye as builder

WORKDIR /usr/src/ic-bot

# Copia solo i file necessari per il caching delle dipendenze
COPY Cargo.toml Cargo.lock ./

# Crea un progetto fittizio per compilare le dipendenze
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm src/main.rs

# Copia il resto del codice sorgente
COPY . .

# Tocca main.rs per forzare la ricompilazione
RUN touch src/main.rs

# Compila l'applicazione
RUN cargo build --release

# Immagine finale
FROM debian:buster-slim

# Copia solo l'eseguibile dall'immagine builder
COPY --from=builder /usr/src/ic-bot/target/release/ic-bot /usr/local/bin/ic-bot

CMD ["ic-bot"]