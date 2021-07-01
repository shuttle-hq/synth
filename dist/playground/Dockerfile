FROM rustlang/rust:nightly-buster

WORKDIR .

COPY core/ core/
COPY gen/ gen/
COPY dist/playground dist/playground

RUN cargo install --path dist/playground

ENV RUST_LOG="debug"

ENTRYPOINT ["synth-playground", "serve", "--port", "8080", "--addr", "0.0.0.0"]