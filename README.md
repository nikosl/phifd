# Phi accrual failure detection demo

Implementation of [The Phi Accrual Failure Detector](https://pdfs.semanticscholar.org/11ae/4c0c0d0c36dc177c1fff5eb84fa49aa3e1a8.pdf) by Hayashibara et al based on Akka.

## Start demo

requirements:

* docker
* docker-compose
* jq
* (optional) [websocat](https://github.com/vi/websocat)

``` bash
make build
make up
make register
websocat ws://localhost:8001/ws/ | jq '.' # or visit http://localhost:8001
```

### Todo

* [ ] tests
  * [ ] latency
  * [ ] packet drops
* [ ] charts
