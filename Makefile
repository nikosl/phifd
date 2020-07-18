SHELL := /bin/bash
THIS_FILE := $(lastword $(MAKEFILE_LIST))

OPEN = xdg-open
CURL = curl --max-time 0.5 --connect-timeout 0.5 -s

PEERS = 8000 8001 8002 8003

TESTER = docker run -it --rm  -v /var/run/docker.sock:/var/run/docker.sock gaiaadm/pumba
TESTER_FLAGS ?= --random --log-level=info
FILTER_PEERS ?= peer[1-9]+[0-9]*
TEST_DURATION ?= 30s
DELAY ?= 2000
PERCENT ?= 40

.PHONY: help build up down destroy stop restart logs ps register unregister show test-pause test-delay test-loss test-stress

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
	LC_NUMERIC=C; \
	for _this in $(PEERS); do \
		_peer_info=$$($(CURL) -H "Accept: application/json" "http://localhost:$$_this/api/info" || echo ""); \
		for _to in $(PEERS); do \
			if test -z $$_peer_info || test $$_this == $$_to; then \
				continue; \
			fi; \
			$(CURL) -H "Content-Type: application/json" -X POST -d $$_peer_info "http://localhost:$$_to/api/register" || echo "error register $$_this to $$_to"; \
		done; \
	done

unregister:
	LC_NUMERIC=C; \
	for _this in $(PEERS); do \
		_id=$$($(CURL) -H "Accept: application/json" "http://localhost:$$_this/api/info" | jq -r '.id' || echo ""); \
		for _from in $(PEERS); do \
			if test -z $$_id || test $$_this == $$_from; then \
				continue; \
			fi; \
			$(CURL) -X DELETE "http://localhost:$$_from/api/unregister/$$_id" || echo "error unregister $$_this from $$_from"; \
		done; \
	done

show:
	$(OPEN) http://localhost:8000/

test-pause:
	$(TESTER) $(TESTER_FLAGS) pause --duration $(TEST_DURATION) "re2:$(FILTER_PEERS)"

test-delay:
	$(TESTER) $(TESTER_FLAGS) netem --tc-image gaiadocker/iproute2 --duration $(TEST_DURATION)\
		delay --time $(DELAY) "re2:$(FILTER_PEERS)"

test-loss:
	$(TESTER) $(TESTER_FLAGS) netem --tc-image gaiadocker/iproute2 --duration $(TEST_DURATION)\
		loss --percent $(PERCENT) "re2:$(FILTER_PEERS)"

test-stress:
	$(TESTER) $(TESTER_FLAGS) stress --pull-image --duration $(TEST_DURATION) "re2:$(FILTER_PEERS)"
