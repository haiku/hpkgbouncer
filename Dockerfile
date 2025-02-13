FROM docker.io/rust:1.84

WORKDIR /usr/srv/hpkgbouncer
COPY . .

RUN cargo install --path . \
	&& rm -rf *

EXPOSE 8000
CMD ["hpkgbouncer"]
