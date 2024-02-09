

source .env;

db_file="${DATABASE_URL#'sqlite://'}";
iso_date="$(date -Iseconds)";
backup_file="${db_file%'.sqlite'}-${iso_date}-backup.sqlite";

echo "Backing up <${db_file}> in <${backup_file}>…" >&2
cp "${db_file}" "${backup_file}";

sea-orm-cli migrate fresh;
