default:
	cargo clean
	docker build --no-cache --tag docker.io/haiku/hpkgbouncer:0.2.2 .
push:
	docker push docker.io/haiku/hpkgbouncer:0.2.2
