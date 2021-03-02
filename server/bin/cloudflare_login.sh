#!/bin/sh

docker run \
	--mount type=bind,src=/opt/cloudflared-secret,dst=/home/nonroot/.cloudflared \
	cloudflare/cloudflared login
