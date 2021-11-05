#!/bin/bash

cd database
../target/release/sqlx mig run
