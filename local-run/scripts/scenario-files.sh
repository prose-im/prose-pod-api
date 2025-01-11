# NOTE: The `: ${VAR_NAME:=value}` syntax initializes a variable only if it's unset or null.
#   It avoids resetting a variable when sourcing this file after the variable was overriden.

: ${SCENARIO_NAME:="${DEFAULT_SCENARIO_NAME:?}"}
: ${SCENARIO_DIR:="${LOCAL_RUN_DIR:?}"/scenarios/"${SCENARIO_NAME:?}"}
: ${DATABASE_PATH:="${SCENARIO_DIR:?}"/database.sqlite}
: ${PROSE_CONFIG_FILE:="${SCENARIO_DIR:?}"/Prose.toml}
: ${ENV_FILE:="${SCENARIO_DIR:?}"/local-run.env}
: ${ETC_PROSODY_DIR:="${SCENARIO_DIR:?}"/prosody/config}
: ${VAR_LIB_PROSODY_DIR:="${SCENARIO_DIR:?}"/prosody/data}
: ${COREDNS_COREFILE:="${SCENARIO_DIR:?}"/coredns/Corefile}
: ${DNS_ZONE_FILE:="${SCENARIO_DIR:?}"/dns-zone.zone}
