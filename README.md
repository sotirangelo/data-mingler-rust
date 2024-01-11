# DataMingler written in Rust [![Rust CI](https://github.com/sotirangelo/data-mingler-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/sotirangelo/data-mingler-rust/actions/workflows/ci.yml) [![Test Coverage](https://raw.githubusercontent.com/sotirangelo/data-mingler-rust/_xml_coverage_reports/data/main/badge.svg)](https://raw.githubusercontent.com/sotirangelo/data-mingler-rust/_xml_coverage_reports/data/main/badge.svg)

DataMingler is a tool that implements Data Virtual Machines (DVMs).
DVM is a graph-based conceptual model that serves as a representation of data, treating
entities and attributes symmetrically and thus allowing for more flexible data manipulation.

This repository contains the source code of the DataMingler engine, written in Rust.

## Pre-requisites

- [Rust](https://www.rust-lang.org/tools/install)

## Build

All the following commands should be run from the root of the repository.

```
cargo build
```

## Run

```
cargo run <path-to-datasources> <path-to-query>
```

### Arguments

- `path-to-datasources`: path to the XML file containing the definition of the datasources

- `path-to-query`: path to the XML file containing the definition of the query you wish to run

- `--output [NONE|EXCEL|CSV]`: (optional) output format. Default: NONE

- `--mode [ALL|INTERSECT]`: (optional) whether to include all rows or only the intersecting ones. Default: ALL

## Test

```
cargo test
```
