#!/bin/sh
# SPDX-License-Identifier: AGPL-3.0-or-later
set -eu

mkdir -p /var/lib/gch/jobs

exec "$@"
