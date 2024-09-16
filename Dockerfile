FROM docker.io/rust:1.81

WORKDIR /usr/srv/hpkgbouncer
COPY . .

RUN cargo install --path . \
	&& rm -rf *

EXPOSE 8000
CMD ["hpkgbouncer"]

HEALTHCHECK --start-period=5m CMD curl --fail http://`hostname`:8000/ || exit 1
