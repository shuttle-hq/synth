FROM rustlang/rust:nightly-buster

WORKDIR .

COPY core/ core/
COPY gen/ gen/
COPY dist/playground dist/playground

RUN cargo install --path dist/playground

ENTRYPOINT ["synth-playground", "serve"]