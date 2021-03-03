#!/bin/sh

if [ ! -e .config/default.yml ]; then
	echo "url: https://social.namachan10777.dev" >> .config/default.yml
	echo "port: 3000" >> .config/default.yml
	echo "db:" >> .config/default.yml
	echo "  host: db" >> .config/default.yml
	echo "  port: 5432" >> .config/default.yml
	echo "  db: "$POSTGRES_DB >> .config/default.yml
	echo "  user: "$POSTGRES_USER >> .config/default.yml
	echo "  pass: "$POSTGRES_PASSWORD >> .config/default.yml
	echo "redis:" >> .config/default.yml
	echo "  host: redis" >> .config/default.yml
	echo "  port: 6379" >> .config/default.yml
	echo "elasticsearch:" >> .config/default.yml
	echo "  host: es" >> .config/default.yml
	echo "  port: 9200" >> .config/default.yml
	echo "  ssl: false" >> .config/default.yml
	echo "  user: elastic">> .config/default.yml
	echo "  pass: "$ELASTICSEARCH_PASSWORD >> .config/default.yml
	echo "id: aid" >> .config/default.yml
fi
