VERSION ?= 0.3.3
REGISTRY ?= ghcr.io/haiku
default:
	cargo clean
	docker build --no-cache --tag ${REGISTRY}/hpkgbouncer:$(VERSION) .
push:
	docker push ${REGISTRY}/hpkgbouncer:$(VERSION)
enter:
	docker run -it ${REGISTRY}/hpkgbouncer:$(VERSION) /bin/bash -l
test:
	docker run -v /home/kallisti5/secrets-mount:/run/secrets -P ${REGISTRY}/hpkgbouncer:$(VERSION)
