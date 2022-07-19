VERSION ?= 0.3.1
default:
	cargo clean
	docker build --no-cache --tag docker.io/haiku/hpkgbouncer:$(VERSION) .
push:
	docker push docker.io/haiku/hpkgbouncer:$(VERSION)
test:
	docker run -v /home/kallisti5/secrets-mount:/run/secrets -P docker.io/haiku/hpkgbouncer:$(VERSION)
