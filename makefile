tmux:
	./tmux.sh

cli:
	cargo run --bin mir --

oxi:
	cargo run --bin dv-oxi -- -c ./configs/local_oxi.yaml

dizer:
	cargo run --bin dv-dizer -- -c ./configs/local_dizer.yaml

flux:
	cargo run --bin iot-flux -- -c ./configs/local_flux.yaml

redox:
	cargo run --bin iot-redox -- -c ./configs/local_redox.yaml

swarmer:
	cargo run --bin iot-swarmer -- -c ./configs/local_swarmer.yaml

rabbit:
	docker stop rabbitmq || true
	docker run --rm --name rabbitmq -p 5672:5672 -p 15672:15672 rabbitmq:3.12-management

db:
	docker stop surrealdb || true
	docker run --rm --pull always --name surrealdb -p 80:8000 -v ./surrealdb:/opt/surrealdb/ surrealdb/surrealdb:1.0.0-beta.11 start --log info --user root --pass root file:/opt/surrealdb/iot.db
