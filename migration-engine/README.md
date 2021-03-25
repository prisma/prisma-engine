# Prisma Migration Engine

This directory contains the crates that belong to the migration engine behind
[prisma-migrate](https://www.prisma.io/docs/concepts/components/prisma-migrate).

The code and documentation for the executable binary are in the [cli](./cli)
directory.

The core logic shared across connectors is in the [core](./core) directory.

The connector interface and the built-in connectors are in the
[connectors](./connectors) directory.
