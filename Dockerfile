FROM rustlang/rust:nightly

WORKDIR /usr/srv/hpkgbouncer
COPY . .

RUN cargo install --path .

CMD ["hpkgbouncer"]
