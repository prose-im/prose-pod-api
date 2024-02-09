#!/bin/bash

URL='http://127.0.0.1:8000/api-docs/swagger-ui'
if command -v xdg-open &>/dev/null; then
    xdg-open "$URL" &
elif command -v open &>/dev/null; then
    open "$URL" &
else
    echo "Neither xdg-open nor open command found. Cannot open URL."
    exit 1
fi

cargo run
