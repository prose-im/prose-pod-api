# NOTE: The `: ${VAR_NAME:=value}` syntax initializes a variable only if it's unset or null.
#   It avoids resetting a variable when sourcing this file after the variable was overriden.

: ${SCENARIO_NAME:="${DEFAULT_SCENARIO_NAME:?}"}
: ${SCENARIOS_DIR:="${LOCAL_RUN_DIR:?}"/scenarios}
: ${SCENARIO_DIR:="${SCENARIOS_DIR:?}/${SCENARIO_NAME:?}"}

: ${DATABASE_PATH:="${SCENARIO_DIR:?}"/database.sqlite}
: ${MAILPIT_DATABASE_PATH:="${SCENARIO_DIR:?}"/mailpit-database.db}
: ${PROSE_CONFIG_FILE:="${SCENARIO_DIR:?}"/prose.toml}
: ${SCENARIO_CONSTANTS_FILE:="${SCENARIO_DIR:?}"/constants.sh}
: ${ENV_FILE:="${SCENARIO_DIR:?}"/local-run.env}
: ${ETC_PROSODY_DIR:="${SCENARIO_DIR:?}"/prosody/config}
: ${VAR_LIB_PROSODY_DIR:="${SCENARIO_DIR:?}"/prosody/data}
: ${COREDNS_COREFILE:="${SCENARIO_DIR:?}"/coredns/Corefile}
: ${DNS_ZONE_FILE:="${SCENARIO_DIR:?}"/dns-zone.zone}
: ${OTEL_CONFIG_FILE:="${LOCAL_RUN_DIR:?}"/otel-collector-config.yaml}
