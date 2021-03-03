FROM node:14.15.1-alpine AS base

ENV NODE_ENV=production

WORKDIR /misskey

FROM base AS builder

RUN apk add --no-cache \
	autoconf \
	automake \
	file \
	git \
	g++ \
	gcc \
	libc-dev \
	libtool \
	make \
	nasm \
	pkgconfig \
	python \
	zlib-dev

RUN git clone -b master --depth=1 https://github.com/syuilo/misskey.git /misskey
RUN yarn install && yarn build

FROM base AS runner


RUN apk add --no-cache \
	ffmpeg \
	tini
RUN npm i -g web-push
ENTRYPOINT ["/sbin/tini", "--"]

COPY generate_default.sh .
COPY --from=builder /misskey . 

CMD [ "/bin/sh", "-c", "sh generate_default.sh && npm run migrateandstart" ]
