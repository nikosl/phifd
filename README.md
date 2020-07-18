# Phi accrual failure detection demo

Implementation of [The Phi Accrual Failure Detector](https://pdfs.semanticscholar.org/11ae/4c0c0d0c36dc177c1fff5eb84fa49aa3e1a8.pdf) by Hayashibara et al based on Akka.

## Start demo

requirements:

* make
* curl
* [docker](https://docs.docker.com/engine/)
* [docker-compose](https://docs.docker.com/compose/)
* [jq](https://stedolan.github.io/jq/)

Run :

``` bash
make build
make up
make register
make show
make test-pause
```
