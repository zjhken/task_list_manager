#!/usr/bin/env bash

set -Eeu


usage() {
  cat <<EOF
Usage examples: 
$(basename "${BASH_SOURCE[0]}") [-h] [-v] 
$(basename "${BASH_SOURCE[0]}") add "task message" "2021-11-21"
$(basename "${BASH_SOURCE[0]}") list
$(basename "${BASH_SOURCE[0]}") list --expiring-today
$(basename "${BASH_SOURCE[0]}") done 1

Script description here.

Available options:

-h, --help      Print this help and exit
-v, --verbose   Print script debug info

sub task:
    add <task message> <date string in rfc3339 format> : Add new task.
    list [--expiring-today]: list all task.
    done <task id>: mark a task as done.
EOF
  exit
}

parse_params() {
  # default values of variables set from params
  taskMessage=''
	dateStr=''
	expiringToday='N'
	taskId=''

  while :; do
    case "${1-}" in
    -h | --help) usage ;;
    -v | --verbose) set -x ;;
    -p | --param) # example named parameter

      shift
      ;;
		add)
		  taskMessage="${2-}"
			dateStr="${3-}"
			add "$taskMessage" "$dateStr"
			exit 0
		;;
		list)
			if [[ "${2-}" == '--expiring-today' ]]; then
				expiringToday=Y
			fi
			list "$expiringToday"
			exit 0
		;;
		done)
			taskId="${2-}"
			markDone "$taskId" 
			exit 0
		;;
    -?*) die "Unknown option: $1" ;;
    *) break ;;
    esac
    shift
  done

  args=("$@")

  # check required params and arguments
  [[ -z "${param-}" ]] && die "Missing required parameter: param"
  [[ ${#args[@]} -eq 0 ]] && die "Missing script arguments"

  return 0
}

function list {
	curl --request GET --url http://localhost:8800/list --header 'content-type: application/json' --header 'user-agent: vscode-restclient' --data '{"today": "'"$1"'"}'
}

function add {
	curl --request POST --url http://localhost:8800/add --header 'content-type: application/json' --header 'user-agent: vscode-restclient' --data '{"task": "'"$1"'","date": "'"$2"'"}'
}

function markDone {
	curl --request PUT --url http://localhost:8800/done --header 'content-type: application/json' --header 'user-agent: vscode-restclient' --data '{"num": '"$1"'}'
}


parse_params "$@"

