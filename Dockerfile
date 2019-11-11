FROM rustlang/rust:nightly

WORKDIR /usr/srv/hpkgserve
COPY . .

RUN cargo install --path .

CMD ["reposerve"]
