FROM debian:buster-slim

WORKDIR /tmp

COPY build-deps.txt /tmp
COPY setup.sh /tmp

RUN /tmp/setup.sh

WORKDIR /root
ENTRYPOINT [""]
