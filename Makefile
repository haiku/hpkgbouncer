default:
	docker build --no-cache --tag quay.io/kallisti5/reposerv:latest .
push:
	docker push quay.io/kallisti5/reposerv:latest
