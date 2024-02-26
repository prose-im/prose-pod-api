open-swagger-ui:
	@(./scripts/open-swagger-ui.sh)
reset-db:
	@(./scripts/reset-db.sh)
entities: reset-db
	rm entity/src/*
	sea-orm-cli generate entity -o entity/src --lib \
		--with-serde both \
		--serde-skip-deserializing-primary-key \
		--serde-skip-hidden-column
		# --model-extra-derives 'rocket::form::FromForm'
format-all:
	cargo fmt
format:
	@(./.githooks/pre-commit)
test:
	cargo test --test cucumber
