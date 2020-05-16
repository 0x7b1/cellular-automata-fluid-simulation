FROM rust:1.31

WORKDIR /usr/src/automata-sandbox
COPY . .

RUN cargo install --path .

CMD ["automata-sandbox"]
