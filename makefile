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

db:
	docker run --rm --pull always -p 80:8000 -v ./surrealdb:/opt/surrealdb/ surrealdb/surrealdb:latest start --log trace --user root --pass root file:/opt/surrealdb/iot.db
