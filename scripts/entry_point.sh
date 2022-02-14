#!/usr/bin/env bash

./battlemon_indexer run | jq '{n: .name, m: .msg}'
