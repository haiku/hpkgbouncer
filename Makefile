VERSION ?= 0.4.2
REGISTRY ?= ghcr.io/haiku
default:
	cargo clean
	docker build --no-cache --tag ${REGISTRY}/hpkgbouncer:$(VERSION) .
push:
	docker push ${REGISTRY}/hpkgbouncer:$(VERSION)
enter:
	docker run -it ${REGISTRY}/hpkgbouncer:$(VERSION) /bin/bash -l
test:
	docker run -e ROCKET_LOG_LEVEL=debug -v /home/kallisti5/secrets-mount:/run/secrets -P ${REGISTRY}/hpkgbouncer:$(VERSION)
