RUNNER_IMG_NAME=pentoxide-runner
SPHINX_IMG_NAME=sphinxdoc/sphinx-latexpdf
DOCKERFILE=kernel/Dockerfile
DOCKER_DIR=`dirname $(DOCKERFILE)`
CARGO_PROJ=`pwd`/kernel
SPHINX_DIR=`pwd`/docs


# === BAREMETAL KERNEL RULES ===

build:
	(cd $(CARGO_PROJ) && cargo build)

run:
	(cd $(CARGO_PROJ) && cargo run)

release:
	(cd $(CARGO_PROJ) && cargo build --release)

clean:
	(cd $(CARGO_PROJ) && cargo clean)


# === DOCKER KERNEL RULES ===

docker-runner-img:
	sudo docker build -t $(RUNNER_IMG_NAME) $(DOCKER_DIR)

docker-runner: docker-runner-img
	sudo docker run \
		--rm \
		--mount type=bind,source=$(CARGO_PROJ),target=/cargo \
		$(RUNNER_IMG_NAME)

docker-runner-clean: docker-runner-img
	sudo docker run \
		--rm \
		--mount type=bind,source=$(CARGO_PROJ),target=/cargo \
		$(RUNNER_IMG_NAME) clean


# === BAREMETAL DOCS RULES ===

docs:
	(cd $(SPHINX_DIR) && make latexpdf)
	(cd $(SPHINX_DIR) && make html)

docs-clean:
	(cd $(SPHINX_DIR) && make clean)


# === DOCKER DOCS RULES ===

docker-docs:
	sudo docker run \
		--rm \
		--mount type=bind,source=$(SPHINX_DIR),target=/docs \
		$(SPHINX_IMG_NAME) make html
	sudo docker run \
		--rm \
		--mount type=bind,source=$(SPHINX_DIR),target=/docs \
		$(SPHINX_IMG_NAME) make latexpdf

docker-docs-clean:
	sudo docker run \
		--rm \
		--mount type=bind,source=$(SPHINX_DIR),target=/docs \
		$(SPHINX_IMG_NAME) make clean
