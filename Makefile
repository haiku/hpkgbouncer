default:
	cargo clean
	docker build --no-cache --tag docker.io/haiku/hpkgbouncer:0.3.0 .
push:
	docker push docker.io/haiku/hpkgbouncer:0.3.0
test:
	docker run -v /home/kallisti5/secrets-mount:/run/secrets -P docker.io/haiku/hpkgbouncer:0.3.0
