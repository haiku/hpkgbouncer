VERSION ?= 0.4.3
REGISTRY ?= ghcr.io/haiku
ENGINE ?= podman
default:
	cargo clean
	${ENGINE} build --no-cache --tag ${REGISTRY}/hpkgbouncer:$(VERSION) .
push:
	${ENGINE} push ${REGISTRY}/hpkgbouncer:$(VERSION)
enter:
	${ENGINE} run -it ${REGISTRY}/hpkgbouncer:$(VERSION) /bin/bash -l
test:
	${ENGINE} run -e ROCKET_LOG_LEVEL=debug -e ROCKET_ADDRESS=0.0.0.0 -v /home/kallisti5/secrets-mount:/run/secrets -P ${REGISTRY}/hpkgbouncer:$(VERSION)
