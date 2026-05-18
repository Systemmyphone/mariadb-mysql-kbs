#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_DIR="${SCRIPT_DIR}/../data/variables"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

ok=0
redirected=0
failed=0
total=0

printf "%-65s | %-6s | %s\n" "File" "Status" "URL / Final URL"
printf '%s\n' "$(printf '%.0s-' {1..140})"

for json_file in "${DATA_DIR}"/mariadb-*.json; do
    filename="$(basename "${json_file}")"
    url="$(grep -o '"url": *"[^"]*"' "${json_file}" | head -1 | sed 's/"url": *"\(.*\)"/\1/')"

    if [[ -z "${url}" ]]; then
        printf "%-65s | %-6s | %s\n" "${filename}" "SKIP" "(no url field)"
        continue
    fi

    total=$((total + 1))

    http_info="$(curl -s -o /dev/null -w "%{http_code} %{url_effective}" -L --max-redirs 10 --connect-timeout 10 --max-time 20 "${url}" 2>/dev/null || echo "000 ERROR")"
    http_code="${http_info%% *}"
    final_url="${http_info#* }"

    if [[ "${http_code}" == "200" ]]; then
        if [[ "${final_url}" == "${url}" ]]; then
            printf "${GREEN}%-65s | %-6s | %s${NC}\n" "${filename}" "${http_code}" "OK"
            ok=$((ok + 1))
        else
            printf "${YELLOW}%-65s | %-6s | redirected -> %s${NC}\n" "${filename}" "${http_code}" "${final_url}"
            redirected=$((redirected + 1))
        fi
    elif [[ "${http_code}" == "000" ]]; then
        printf "${RED}%-65s | %-6s | %s${NC}\n" "${filename}" "ERROR" "${url}"
        failed=$((failed + 1))
    else
        printf "${RED}%-65s | %-6s | %s${NC}\n" "${filename}" "${http_code}" "${url}"
        failed=$((failed + 1))
    fi
done

printf '%s\n' "$(printf '%.0s-' {1..140})"
printf "\nTotal: %d  |  ${GREEN}OK: %d${NC}  |  ${YELLOW}Redirected: %d${NC}  |  ${RED}Failed: %d${NC}\n\n" \
    "${total}" "${ok}" "${redirected}" "${failed}"