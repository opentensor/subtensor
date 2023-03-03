#!/usr/bin/env bash

## Find true directory this script resides in
__SOURCE__="${BASH_SOURCE[0]}"
while [[ -h "${__SOURCE__}" ]]; do
    __SOURCE__="$(find "${__SOURCE__}" -type l -ls | sed -n 's@^.* -> \(.*\)@\1@p')"
done
__NAME__="${__SOURCE__##*/}"
__DIR__="$(cd -P "$(dirname "${__SOURCE__}")" && pwd)"
__G_DIR__="$(dirname "${__DIR__}")"
__AUTHOR__='S0AndS0'
__DESCRIPTION__='Generate and display code coverage data'

set -eET

## TODO: maybe consider using package from `cargo install toml`
KNOWN_MEMBERS=(
  node
  pallets/subtensor
  runtime
)

## Defaults
EXTRA_ARGUMENTS=()
MEMBER=''
VERBOSE=0

##
#
__version__() {
  local _version_number
  _version_number="$(git tag --list v* | tail -n1)"
  _version_number="${_version_number:-v0.1.0}"
  _version_number="${_version_number//[^0-9.]/}"

  printf '%s - %s\n' "${__NAME__}" "${_version_number}"
  __license__ "${__AUTHOR__}"
  printf '\nWritten by %s\n' "${__AUTHOR__}"
}

##
#
__license__(){
  local __AUTHOR__="${__AUTHOR__:-S0AndS0}"
  local _year
  _year="$(date +'%Y')"
  cat <<EOF
Copyright (C) ${_year} ${__AUTHOR__}

$(< "${__G_DIR__}/LICENSE")
EOF
}

##
#
__usage__() {
  local _message="${1}"
  local _status="${2:-0}"

  cat <<EOF
${__DESCRIPTION__}


Usage: ${__NAME__} [OPTIONS]...


Options:
--help
    Prints this message and exits

--license
    Prints copyright for this script and exits

--verbose
    Prints messages about actions

--version
    Prints version for this script and exits

--member <PATH>
    Sub-directory or source code path to run spicific tests for

Examples:
    ${__NAME__} --verbose --member ${KNOWN_MEMBERS[0]}
EOF

  if (( ${#_message} )); then
    printf >&2 '\n\nError: %s\n' "${_message}"
    _status=1
  fi

  exit "${_status}"
}

while (( ${#@} )); do
  case "${1}" in
    --)
      shift 1
      EXTRA_ARGUMENTS+=( "${@}" )
      break
    ;;
    --member)
      if grep -qE "\<${2:?Undefined --member}\>" <<<"${KNOWN_MEMBERS[@]}"; then
        MEMBER="${2}"
      else
        __usage__ 'Unknown --member' 1
      fi
      shift 2
    ;;
    --verbose)
      VERBOSE=1
      shift 1
    ;;
    --license)
      __license__
      exit 0
    ;;
    --version)
      __version__
      exit 0
    ;;
    --help)
      __usage__ '' 0
    ;;
  esac
done

##
#
run_tests() {
  local _member="${1}"
  local _args=( test )

  if (( ${#_member} )); then
    _args+=( "${_member}" )
  else
    _args+=( --tests )
  fi

  if ((VERBOSE)); then
    printf >&2 'cargo %s\n' "${_args[*]}"
  fi

  SKIP_WASM_BUILD="${SKIP_WASM_BUILD:-1}" \
    RUST_LOG="${RUST_LOG:-runtime=debug}" \
    RUSTFLAGS="-C instrument-coverage" \
    cargo "${_args[@]}"
}

##
#
merge_profraw_files() {
  local _member="${1}"

  local _args=( merge --sparse "${__G_DIR__}"/default_*.profraw )

  if (( ${#_member} )); then
    _args+=( -o "${__G_DIR__}/coverage_${_member//\/-}.profdata" )
  else
    _args+=( -o "${__G_DIR__}/coverage.profdata" )
  fi

  # if ((VERBOSE)); then
  #   printf >&2 'llvm-profdata %s\n' "${_args[*]}"
  # fi

  llvm-profdata "${_args[@]}"
}

##
#
show_coverage_report() {
  local _member="${1}"

  local _args=(
    show
    -Xdemangler=rustfilt 
    --show-line-counts-or-regions
    --show-instantiations
  )

  if (( ${#_member} )); then
    _args+=( --instr-profile="${__G_DIR__}/coverage_${_member//\/-}.profdata" )
  else
    _args+=( --instr-profile="${__G_DIR__}/coverage.profdata" )
  fi

  if ((VERBOSE)); then
    printf >&2 'livm-cov %s\n' "${_args[*]}"
  fi

  printf >&2 'TODO: fully implement showing code coverage reports\n'
  return 1
  # llvm-cov "${_args[@]}"
}

##
# Do the things!
run_tests "${MEMBER}"
merge_profraw_files "${MEMBER}"
show_coverage_report "${MEMBER}"


## Attributions
# https://doc.rust-lang.org/rustc/instrument-coverage.html
