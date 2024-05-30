open-swagger-ui:
	@(./scripts/open-swagger-ui.sh)
reset-db:
	@(./scripts/reset-db.sh)
entity: reset-db
	sea-orm-cli generate entity -o entity/src --lib \
		--tables "$(TABLES)" \
		--with-serde both \
		--serde-skip-deserializing-primary-key \
		--serde-skip-hidden-column
		# --model-extra-derives 'rocket::form::FromForm'
format-all:
	cargo fmt
format:
	@(./.githooks/pre-commit)
test: smoke-test integration-test
smoke-test:
	cargo test --test behavior
integration-test:
	@(./scripts/integration-test.sh)
update: update-redoc
# NOTE: `cargo update` updates all workspace member crates
	@(echo '[INFO] Updating Rust dependencies…')
	@(cargo update)

# Check for outdated dependencies
	@(if cargo install --list | grep -q '^cargo-edit v'; then \
		echo '[INFO] Checking for outdated dependencies…'; \
		cargo upgrade --dry-run --incompatible --pinned --verbose; \
	else \
		echo '[WARN] Install `cargo upgrade` with `cargo install cargo-edit` for this script to check for outdated dependencies'; \
	fi)
update-redoc:
	@(echo '[INFO] Updating Redoc…')
	@(wget -q https://cdn.redoc.ly/redoc/latest/bundles/redoc.standalone.js -O static/api-docs/redoc.standalone.js)
%:
	@:
