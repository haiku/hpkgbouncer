VERSION ?= 0.3.2
default:
	cargo clean
	docker build --no-cache --tag docker.io/haiku/hpkgbouncer:$(VERSION) .
push:
	docker push docker.io/haiku/hpkgbouncer:$(VERSION)
enter:
	docker run -it docker.io/haiku/hpkgbouncer:$(VERSION) /bin/bash -l
test:
	docker run -v /home/kallisti5/secrets-mount:/run/secrets -P docker.io/haiku/hpkgbouncer:$(VERSION)
