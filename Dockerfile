FROM ubuntu:18.04

RUN apt-get update && \
	apt-get install -y software-properties-common && \
	add-apt-repository ppa:longsleep/golang-backports && \
	apt-get update && \
	apt-get upgrade -y && \
	apt-get install -y fish curl git make golang-go

ENTRYPOINT [ "/usr/bin/fish" ]
