#!/bin/sh
set -e

mkdir -p /data/storage /data/backups
chown -R appuser:appuser /data/storage
chown appuser:appuser /data/backups

exec gosu appuser "$@"
