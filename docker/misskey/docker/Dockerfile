FROM node:14.16-buster

ENV NODE_ENV=production

WORKDIR /misskey

RUN git clone -b master --depth=1 https://github.com/syuilo/misskey.git /misskey
RUN yarn install && yarn build
RUN apt-get update && apt-get install -y ffmpeg
RUN touch /misskey/.config/default.yml

COPY startup.mjs .

CMD [ "startup.mjs" ]
