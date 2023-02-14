ifeq (run,$(firstword $(MAKECMDGOALS)))
  RUN_ARGS := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
  $(eval $(RUN_ARGS):;@:)
endif
ifeq (docker-run,$(firstword $(MAKECMDGOALS)))
  RUN_ARGS := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
  $(eval $(RUN_ARGS):;@:)
endif

.PHONY: run clear build

run: build
	./target/debug/arrow-generator $(RUN_ARGS)

exec:
	./target/debug/arrow-generator

clear:
	sudo ./bin/spectrum clear

build:
	cargo build

docker-build:
	docker build --tag maxthom/mir/arrow-generator:latest -f Dockerfile .

docker-run:
	docker run --rm maxthom/mir/arrow-generator:latest $(RUN_ARGS)

docker-exec:
	docker ex -it --rm maxthom/mir/arrow-generator:latest /bin/sh

docker-push:
	docker push maxthom/mir/arrow-generator:latest

docker-buildx:
	docker buildx build --push --platform linux/arm/v6,linux/arm/v7 -t maxthom/mir/arrow-generator:latest .

deploy:
	docker-compose -f ./build/docker-compose/docker-compose.yaml down
	docker-compose -f ./build/docker-compose/docker-compose.yaml rm -f
	docker-compose -f ./build/docker-compose/docker-compose.yaml pull
	docker-compose -f ./build/docker-compose/docker-compose.yaml up --build -d
	docker-compose -f ./build/docker-compose/docker-compose.yaml logs

deploy-down:
	docker-compose -f ./build/docker-compose/docker-compose.yaml down