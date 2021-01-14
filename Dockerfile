FROM ubuntu:18.04

RUN apt-get update && \
	apt-get install --no-install-recommends -y make=4.1-9.1ubuntu1 && \
	rm /var/lib/apt/lists/*

ENTRYPOINT [ "/usr/bin/fish" ]
