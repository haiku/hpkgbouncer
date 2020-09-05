default:
	docker build --no-cache --tag docker.io/haiku/hpkgbouncer:0.2.0 .
push:
	docker push docker.io/haiku/hpkgbouncer:0.2.0
