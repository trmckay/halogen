RUNNER_IMG_NAME=pentoxide-runner
SPHINX_IMG_NAME=sphinxdoc/sphinx
DOCKERFILE=kernel/Dockerfile
DOCKER_DIR=`dirname $(DOCKERFILE)`
CARGO_PROJ=`pwd`/kernel
SPHINX_DIR=`pwd`/docs


# === BAREMETAL KERNEL RULES ===

init:
	cd $(CARGO_PROJ)
	# configure rust toolchain
	rustup override set nightly
	rustup target add riscv64gc-unknown-none-elf
	# install pre-commit hooks
	eval `pwd`/scripts/install_hooks.sh

build:
	(cd $(CARGO_PROJ) && cargo build)

run:
	(cd $(CARGO_PROJ) && cargo run)

release:
	(cd $(CARGO_PROJ) && cargo build --release)

clean:
	(cd $(CARGO_PROJ) && cargo clean)


# === DOCKER KERNEL RULES ===

run-img-docker:
	sudo docker build -t $(RUNNER_IMG_NAME) $(DOCKER_DIR)

run-docker: run-img-docker
	sudo docker run \
		--rm \
		--mount type=bind,source=$(CARGO_PROJ),target=/cargo \
		$(RUNNER_IMG_NAME)

clean-docker: run-img-docker
	sudo docker run \
		--rm \
		--mount type=bind,source=$(CARGO_PROJ),target=/cargo \
		$(RUNNER_IMG_NAME) clean


# === DOCS RULES ===

docs:
	sudo docker run \
		--rm \
		--mount type=bind,source=$(SPHINX_DIR),target=/docs \
		$(SPHINX_IMG_NAME) make html

docs-clean:
	sudo docker run \
		--rm \
		--mount type=bind,source=$(SPHINX_DIR),target=/docs \
		$(SPHINX_IMG_NAME) make clean
