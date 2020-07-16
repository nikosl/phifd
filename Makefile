SHELL := /bin/bash
THIS_FILE := $(lastword $(MAKEFILE_LIST))
.PHONY: help build up down destroy stop restart logs ps register unregister

help:
	make -pRrq  -f $(THIS_FILE) : 2>/dev/null | awk -v RS= -F: '/^# File/,/^# Finished Make data base/ {if ($$1 !~ "^[#.]") {print $$1}}' | sort | egrep -v -e '^[^[:alnum:]]' -e '^$@$$'
build:
	docker-compose -f docker-compose.yml build $(c)
up:
	docker-compose -f docker-compose.yml up -d $(c)
down:
	docker-compose -f docker-compose.yml down $(c)
destroy:
	docker-compose -f docker-compose.yml down -v $(c)
restart:
	docker-compose -f docker-compose.yml stop $(c)
	docker-compose -f docker-compose.yml up -d $(c)
logs:
	docker-compose -f docker-compose.yml logs --tail=100 -f $(c)
ps:
	docker-compose -f docker-compose.yml ps
register: up
	curl -s -H "Content-Type: application/json" -X GET http://localhost:8000/api/info | curl -s -H "Content-Type: application/json" -X POST -d @- http://localhost:8001/api/register
	curl -s -H "Content-Type: application/json" -X GET http://localhost:8001/api/info | curl -s -H "Content-Type: application/json" -X POST -d @- http://localhost:8000/api/register
unregister:
	id1=$$(curl -s -H "Content-Type: application/json" -X GET http://localhost:8001/api/info | jq -r '.id')\
	 && curl -s -X DELETE "http://localhost:8000/api/unregister/$$id1"
	id0=$$(curl -s -H "Content-Type: application/json" -X GET http://localhost:8000/api/info | jq -r '.id')\
	 && curl -s -X DELETE "http://localhost:8001/api/unregister/$$id0"