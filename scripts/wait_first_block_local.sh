#!/usr/bin/env bash

DIR=$(dirname "$0")

function get_chain_head {
	curl -s --header "Content-Type: application/json" -XPOST --data "{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"chain_getHeader\",\"params\":[]}" "localhost:9946"
}

last_block_id=0
block_id=0
function get_block {
	block_id_hex=$(get_chain_head | jq -r .result.number)
	block_id=$((block_id_hex))
	echo Id = $block_id
}

function had_new_block {
	last_block_id=$block_id
	get_block
	if (( last_block_id != 0 && block_id > last_block_id )); then
		return 0
	fi
	return 1
}

function reset_check {
	last_block_id=0
	block_id=0
}

while ! had_new_block; do
	echo "Waiting for next block..."
	sleep 6
done
reset_check

echo "Chain is running"

while ! had_new_block; do
	echo "Waiting for another block..."
	sleep 6
done

echo "Chain is definitely running!"
