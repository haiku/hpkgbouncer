FROM rustlang/rust:nightly

WORKDIR /usr/srv/hpkgbouncer
COPY . .

RUN cargo install --path . \
	&& rm -rf *

CMD ["hpkgbouncer"]

HEALTHCHECK --start-period=5m CMD curl --fail http://`hostname`:8000/ || exit 1
