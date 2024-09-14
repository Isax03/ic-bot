ARG teloxide_token

FROM rust:latest

ENV TEL_OXIDE_TOKEN=$teloxide_token
COPY ./ ./

RUN cargo build --release

CMD ["./target/release/ic-bot"]