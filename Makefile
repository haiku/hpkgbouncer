VERSION ?= 0.4.5
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
	${ENGINE} run -e ROCKET_LOG_LEVEL=debug -e ROCKET_ADDRESS=0.0.0.0 -e CACHE_TTL=900 -e S3_PUBLIC="https://haikuports-repository.cdn.haiku-os.org/" -e BRANCH_ALIASES="master:r1beta5,r1beta4,r1beta3" -v ./secrets-mount:/run/secrets -P ${REGISTRY}/hpkgbouncer:$(VERSION)
	#${ENGINE} run -e ROCKET_LOG_LEVEL=debug -e ROCKET_ADDRESS=0.0.0.0 -e CACHE_TTL=900 -e S3_PUBLIC="https://haikuports-repository.cdn.haiku-os.org/" -v ./secrets-mount:/run/secrets -P ${REGISTRY}/hpkgbouncer:$(VERSION)
