FROM debian:buster-slim

WORKDIR /tmp

COPY build-deps.txt /tmp
COPY bootstrap.sh /tmp

RUN /tmp/bootstrap.sh

WORKDIR /root
ENTRYPOINT [""]
