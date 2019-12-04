default:
	docker build --no-cache --tag docker.io/haiku/hpkgbouncer:latest .
push:
	docker push docker.io/haiku/hpkgbouncer:latest
